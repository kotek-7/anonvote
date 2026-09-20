mod db;

use actix_web::HttpResponse;
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
    let cli = Cli::parse();

    let postgres_url = std::env::var("POSTGRES_URL")
        .unwrap_or("postgresql://postgres:postgres@localhost:5432/postgres".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await?;

    match &cli.command {
        Commands::Start => start_server(pool).await?,
    }

    Ok(())
}

async fn start_server(pool: sqlx::Pool<Postgres>) -> std::io::Result<()> {
    let pool = actix_web::web::Data::new(pool);
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(pool.clone())
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
            .service(certificate)
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
    let Ok(key_pair) = KeyPair::generate(&mut DefaultRng, 2048) else {
        return HttpResponse::InternalServerError().finish();
    };
    let (pub_key, sec_key) = (&key_pair.pk, &key_pair.sk);
    if save_key_pair(&key_pair, &pool).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    };
    match pub_key.to_pem() {
        Ok(pub_key_pem) => HttpResponse::Ok().body(pub_key_pem),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(sqlx::FromRow)]
struct RawKeyPair {
    id: i32,
    pub_key: String,
    sec_key: String,
}

async fn save_key_pair(key_pair: &KeyPair, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let pub_key = &key_pair.pk.to_pem()?;
    let sec_key = &key_pair.sk.to_pem()?;
    sqlx::query_as::<_, RawKeyPair>("INSERT INTO key_pairs (pub_key, sec_key) VALUES ($1 $2)")
        .bind(&pub_key)
        .bind(&sec_key)
        .fetch_one(pool)
        .await?;
    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct CertificateRequest {
    /// base64 encoded blinded token
    pub blinded_token: String,
    /// PEM encoded public key
    pub pub_key: String,
}

#[actix_web::post("/api/certificate")]
async fn certificate(
    req: actix_web::web::Json<CertificateRequest>,
    pool: actix_web::web::Data<sqlx::PgPool>,
) -> impl actix_web::Responder {
    let Ok(pub_key) = PublicKey::from_pem(&req.pub_key) else {
        return HttpResponse::BadRequest().finish();
    };
    let Ok(key_pair) = fetch_key_pair(&pub_key, pool.get_ref()).await else {
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(blinded_token_bytes) = STANDARD.decode(&req.blinded_token) else {
        return HttpResponse::BadRequest().finish();
    };
    let Ok(blinded_token_sign) = key_pair.sk.blind_sign(&blinded_token_bytes) else {
        return HttpResponse::InternalServerError().finish();
    };
    let blinded_token_sign_base64 = STANDARD.encode(blinded_token_sign);
    HttpResponse::Ok().body(blinded_token_sign_base64)
}

async fn fetch_key_pair(
    pub_key: &PublicKey,
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<KeyPair> {
    let pub_key = &pub_key.to_pem()?;
    let keypair = sqlx::query_as::<_, RawKeyPair>(" SELECT * FROM key_pairs WHERE pub_key = $1")
        .bind(&pub_key)
        .fetch_one(pool)
        .await?;
    Ok(KeyPair {
        pk: PublicKey::from_pem(&keypair.pub_key)?,
        sk: SecretKey::from_pem(&keypair.sec_key)?,
    })
}
