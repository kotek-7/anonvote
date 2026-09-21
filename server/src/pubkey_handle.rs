use actix_web::HttpResponse;
use blind_rsa_signatures::DefaultRng;
use crate::common::{KeyPair, PublicKey, RawKeyPair, SecretKey};
use sqlx::Postgres;

#[actix_web::post("/api/pubkey")]
pub async fn pubkey(pool: actix_web::web::Data<sqlx::PgPool>) -> impl actix_web::Responder {
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