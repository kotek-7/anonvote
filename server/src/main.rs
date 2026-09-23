mod certificate_handler;
mod common;
mod login_handler;
mod pubkey_handler;

use axum::routing::{get, post};
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

async fn start_server(pool: sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = std::sync::Arc::new(common::AppState { pool });
    let app = axum::Router::new()
        .route("/api/pubkey", post(pubkey_handler::pubkey))
        .route(
            "/api/certificate",
            post(certificate_handler::certificate),
        )
        .route("/api/login", get(login_handler::login))
        .route("/api/login/callback", get(login_handler::login_callback))
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
