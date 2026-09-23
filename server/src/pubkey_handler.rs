use crate::common::{KeyPair, PublicKey, RawKeyPair, SecretKey};
use blind_rsa_signatures::DefaultRng;
use sqlx::Postgres;

pub async fn pubkey(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<crate::common::AppState>>,
) -> Result<String, axum::http::StatusCode> {
    resolve_pub_key(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(Debug, thiserror::Error)]
enum ResolvePubKeyError {
    #[error("failed to fetch key pair: {0}")]
    Fetch(#[from] FetchKeyPairError),
    #[error("failed to generate and save key pair: {0}")]
    GenerateAndSave(#[from] GenerateAndSaveKeyPairError),
    #[error("failed to encode public key: {0}")]
    Encode(#[from] blind_rsa_signatures::Error),
}

async fn resolve_pub_key(pool: &sqlx::PgPool) -> Result<String, ResolvePubKeyError> {
    let key_pair = match fetch_key_pair(pool).await? {
        Some(key_pair) => key_pair,
        None => generate_and_save_key_pair(pool).await?,
    };

    Ok(key_pair.pk.to_pem()?)
}

#[derive(Debug, thiserror::Error)]
enum FetchKeyPairError {
    #[error("database query failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to decode stored key pair: {0}")]
    Decode(#[from] blind_rsa_signatures::Error),
}

async fn fetch_key_pair(pool: &sqlx::Pool<Postgres>) -> Result<Option<KeyPair>, FetchKeyPairError> {
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

#[derive(Debug, thiserror::Error)]
pub enum GenerateAndSaveKeyPairError {
    #[error("key generation failed: {0}")]
    Generate(#[from] blind_rsa_signatures::Error),
    #[error("failed to save key pair: {0}")]
    Save(#[from] SaveKeyPairError),
}

async fn generate_and_save_key_pair(
    pool: &sqlx::Pool<Postgres>,
) -> Result<KeyPair, GenerateAndSaveKeyPairError> {
    let key_pair = KeyPair::generate(&mut DefaultRng, 2048)?;
    save_key_pair(&key_pair, &pool).await?;
    Ok(key_pair)
}

#[derive(Debug, thiserror::Error)]
pub enum SaveKeyPairError {
    #[error("failed to encode key pair: {0}")]
    Encode(#[from] blind_rsa_signatures::Error),
    #[error("database insert failed: {0}")]
    Database(#[from] sqlx::Error),
}

async fn save_key_pair(
    key_pair: &KeyPair,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), SaveKeyPairError> {
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
