use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn init_db() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/test").await?;

    Ok(())
}