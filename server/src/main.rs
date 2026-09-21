mod certificate_handle;
mod common;
mod pubkey_handle;
mod discord_auth;

use crate::certificate_handle::SignBlindTokenError;
use actix_web::{HttpResponse, middleware::Logger};
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

#[actix_web::post("/api/pubkey")]
async fn pubkey(pool: actix_web::web::Data<sqlx::PgPool>) -> impl actix_web::Responder {
    match pubkey_handle::resolve_pub_key(pool.get_ref()).await {
        Ok(pub_key_pem) => HttpResponse::Ok().body(pub_key_pem),
        Err(e) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct CertificateRequest {
    /// Base64-encoded blinded token.
    pub blind_token: String,
    /// PEM-encoded public key.
    pub pub_key: String,
}

#[actix_web::post("/api/certificate")]
async fn certificate(
    req: actix_web::web::Json<CertificateRequest>,
    pool: actix_web::web::Data<sqlx::PgPool>,
) -> impl actix_web::Responder {
    match certificate_handle::sign_blind_token(&req.pub_key, &req.blind_token, pool.get_ref()).await
    {
        Ok(signature) => HttpResponse::Ok().body(signature),
        Err(e @ (SignBlindTokenError::InvalidPublicKey(_) | SignBlindTokenError::Decode(_))) => {
            log::warn!("{e}");
            HttpResponse::BadRequest().finish()
        }
        Err(e @ (SignBlindTokenError::Search(_) | SignBlindTokenError::Sign(_))) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
