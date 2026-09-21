use crate::common::{KeyPair, PublicKey, RawKeyPair, SecretKey};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

#[derive(Debug, thiserror::Error)]
pub enum SignBlindTokenError {
    #[error("invalid public key: {0}")]
    InvalidPublicKey(#[source] blind_rsa_signatures::Error),
    #[error("failed to search key pair: {0}")]
    Search(#[from] SearchKeyPairError),
    #[error("invalid blind token encoding: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("failed to sign blind token: {0}")]
    Sign(#[source] blind_rsa_signatures::Error),
}

pub async fn sign_blind_token(
    pub_key_pem: &str,
    blind_token_base64: &str,
    pool: &sqlx::PgPool,
) -> Result<String, SignBlindTokenError> {
    let pub_key =
        PublicKey::from_pem(pub_key_pem).map_err(SignBlindTokenError::InvalidPublicKey)?;
    let key_pair = search_key_pair(&pub_key, pool).await?;
    let blind_token = STANDARD.decode(blind_token_base64)?;
    let blind_token_sign = key_pair
        .sk
        .blind_sign(&blind_token)
        .map_err(SignBlindTokenError::Sign)?;

    Ok(STANDARD.encode(blind_token_sign))
}

#[derive(Debug, thiserror::Error)]
pub enum SearchKeyPairError {
    #[error("failed to encode public key: {0}")]
    Encode(#[source] blind_rsa_signatures::Error),
    #[error("database query failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to decode stored key pair: {0}")]
    Decode(#[source] blind_rsa_signatures::Error),
}

async fn search_key_pair(
    pub_key: &PublicKey,
    pool: &sqlx::PgPool,
) -> Result<KeyPair, SearchKeyPairError> {
    let pub_key = pub_key.to_pem().map_err(SearchKeyPairError::Encode)?;
    let key_pair = sqlx::query_as!(
        RawKeyPair,
        "SELECT * FROM key_pairs WHERE pub_key = $1",
        &pub_key
    )
    .fetch_one(pool)
    .await?;

    Ok(KeyPair {
        pk: PublicKey::from_pem(&key_pair.pub_key).map_err(SearchKeyPairError::Decode)?,
        sk: SecretKey::from_pem(&key_pair.sec_key).map_err(SearchKeyPairError::Decode)?,
    })
}
