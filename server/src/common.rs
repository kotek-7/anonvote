use blind_rsa_signatures::{Deterministic, PSS, Sha384};

pub type KeyPair = blind_rsa_signatures::KeyPair<Sha384, PSS, Deterministic>;
pub type PublicKey = blind_rsa_signatures::PublicKey<Sha384, PSS, Deterministic>;
pub type SecretKey = blind_rsa_signatures::SecretKey<Sha384, PSS, Deterministic>;

#[derive(sqlx::FromRow)]
pub struct RawKeyPair {
    #[allow(unused)]
    pub id: i64,
    pub pub_key: String,
    pub sec_key: String,
}