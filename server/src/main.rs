mod certificate_handle;
mod common;
mod discord_auth;
mod pubkey_handle;

use crate::certificate_handle::SignBlindTokenError;
use clap::Parser;
use sqlx::{Postgres, postgres::PgPoolOptions};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    let postgres_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgresql://postgres:postgres@localhost:5432/postgres".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await?;
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => start_server(pool).await?,
    }

    Ok(())
}

struct AppState {
    pool: sqlx::Pool<Postgres>,
}

async fn start_server(pool: sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = std::sync::Arc::new(AppState { pool });
    let app = axum::Router::new()
        .route("/api/pubkey", axum::routing::post(pubkey))
        .route("/api/certificate", axum::routing::post(certificate))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        std::env::var("API_HOST").unwrap_or("127.0.0.1".to_string()),
        std::env::var("API_PORT").unwrap_or("8081".to_string())
    ))
    .await?;

    axum::serve(listener, app).await.map_err(Into::into)
}

async fn pubkey(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
) -> Result<String, axum::http::StatusCode> {
    pubkey_handle::resolve_pub_key(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct CertificateRequest {
    /// Base64-encoded blinded token.
    pub blind_token: String,
    /// PEM-encoded public key.
    pub pub_key: String,
}

async fn certificate(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
    axum::Json(req): axum::Json<CertificateRequest>,
) -> Result<String, axum::http::StatusCode> {
    certificate_handle::sign_blind_token(&req.pub_key, &req.blind_token, &state.pool)
        .await
        .map_err(|e| match e {
            e @ (SignBlindTokenError::InvalidPublicKey(_) | SignBlindTokenError::Decode(_)) => {
                tracing::warn!("{e}");
                axum::http::StatusCode::BAD_REQUEST
            }
            e @ (SignBlindTokenError::Search(_) | SignBlindTokenError::Sign(_)) => {
                tracing::error!("{e}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
        })
}
