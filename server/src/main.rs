use actix_web::{HttpResponse, middleware::Logger};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use blind_rsa_signatures::{DefaultRng, PSS, Randomized, Sha384};
use clap::Parser;
use sqlx::{Postgres, postgres::PgPoolOptions};

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Start,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    let postgres_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgresql://postgres:postgres@localhost:5432/postgres".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => start_server(pool).await?,
    }

    Ok(())
}

async fn start_server(pool: sqlx::Pool<Postgres>) -> std::io::Result<()> {
    let pool = actix_web::web::Data::new(pool);

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,actix_web=info,actix_server=info"),
    )
    .format_timestamp_millis()
    .init();

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(Logger::default())
            .app_data(pool.clone())
            .service(pubkey)
            .service(certificate)
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

type KeyPair = blind_rsa_signatures::KeyPair<Sha384, PSS, Randomized>;
type PublicKey = blind_rsa_signatures::PublicKey<Sha384, PSS, Randomized>;
type SecretKey = blind_rsa_signatures::SecretKey<Sha384, PSS, Randomized>;

#[actix_web::post("/api/pubkey")]
async fn pubkey(pool: actix_web::web::Data<sqlx::PgPool>) -> impl actix_web::Responder {
    let key_pair = match fetch_key_pair(&pool).await {
        Ok(Some(key_pair)) => key_pair,
        Ok(None) => match generate_and_save_key_pair(&pool).await {
            Ok(key_pair) => key_pair,
            Err(e) => {
                log::error!("{e}");
                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(e) => {
            log::error!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    let (pub_key, _sec_key) = (&key_pair.pk, &key_pair.sk);
    match pub_key.to_pem() {
        Ok(pub_key_pem) => HttpResponse::Ok().body(pub_key_pem),
        Err(e) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(sqlx::FromRow)]
struct RawKeyPair {
    #[allow(unused)]
    id: i64,
    pub_key: String,
    sec_key: String,
}

async fn fetch_key_pair(pool: &sqlx::Pool<Postgres>) -> anyhow::Result<Option<KeyPair>> {
    let key_pair_result = sqlx::query_as!(
        RawKeyPair,
        "SELECT * FROM key_pairs ORDER BY id DESC LIMIT 1"
    )
    .fetch_one(pool)
    .await;

    match key_pair_result {
        Ok(key_pair) => Ok(Some(KeyPair {
            pk: PublicKey::from_pem(&key_pair.pub_key)?,
            sk: SecretKey::from_pem(&key_pair.sec_key)?,
        })),
        Err(e) => match e {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(e.into()),
        },
    }
}

async fn generate_and_save_key_pair(pool: &sqlx::Pool<Postgres>) -> anyhow::Result<KeyPair> {
    let key_pair = KeyPair::generate(&mut DefaultRng, 2048)?;
    save_key_pair(&key_pair, &pool).await?;
    Ok(key_pair)
}

async fn save_key_pair(key_pair: &KeyPair, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let pub_key = &key_pair.pk.to_pem()?;
    let sec_key = &key_pair.sk.to_pem()?;
    sqlx::query!(
        "INSERT INTO key_pairs (pub_key, sec_key) VALUES ($1, $2)",
        &pub_key,
        &sec_key
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct CertificateRequest {
    /// base64 encoded blinded token
    pub blind_token: String,
    /// PEM encoded public key
    pub pub_key: String,
}

#[actix_web::post("/api/certificate")]
async fn certificate(
    req: actix_web::web::Json<CertificateRequest>,
    pool: actix_web::web::Data<sqlx::PgPool>,
) -> impl actix_web::Responder {
    let pub_key = match PublicKey::from_pem(&req.pub_key) {
        Ok(pub_key) => pub_key,
        Err(e) => {
            log::warn!("{e}");
            return HttpResponse::BadRequest().finish();
        }
    };
    let key_pair = match search_key_pair(&pub_key, pool.get_ref()).await {
        Ok(key_pair) => key_pair,
        Err(e) => {
            log::error!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    let blind_token = match STANDARD.decode(&req.blind_token) {
        Ok(blind_token) => blind_token,
        Err(e) => {
            log::warn!("{e}");
            return HttpResponse::BadRequest().finish();
        }
    };
    let blind_token_sign = match key_pair.sk.blind_sign(&blind_token) {
        Ok(blind_token_sign) => blind_token_sign,
        Err(e) => {
            log::error!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    let blind_token_sign_base64 = STANDARD.encode(blind_token_sign);
    HttpResponse::Ok().body(blind_token_sign_base64)
}

async fn search_key_pair(
    pub_key: &PublicKey,
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<KeyPair> {
    let pub_key = &pub_key.to_pem()?;
    let key_pair = sqlx::query_as!(
        RawKeyPair,
        "SELECT * FROM key_pairs WHERE pub_key = $1",
        &pub_key
    )
    .fetch_one(pool)
    .await?;
    Ok(KeyPair {
        pk: PublicKey::from_pem(&key_pair.pub_key)?,
        sk: SecretKey::from_pem(&key_pair.sec_key)?,
    })
}
