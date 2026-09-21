use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use blind_rsa_signatures::{
    BlindMessage, BlindSignature, DefaultRng, Deterministic, PSS, Secret, Sha384, Signature,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

type PublicKey = blind_rsa_signatures::PublicKey<Sha384, PSS, Deterministic>;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlindingResult {
    pub blind_token: String,
    pub secret: String,
}

#[wasm_bindgen]
pub fn blind(token_base64: &str, pub_key_pem: &str) -> Result<JsValue, JsError> {
    let token_bytes = STANDARD.decode(token_base64)?;
    let pub_key: PublicKey = PublicKey::from_pem(pub_key_pem)?;
    let blind_token = pub_key.blind(&mut DefaultRng, token_bytes)?;
    let blind_token_base64 = STANDARD.encode(blind_token.blind_message);
    let secret_base64 = STANDARD.encode(blind_token.secret);
    let result = BlindingResult {
        blind_token: blind_token_base64,
        secret: secret_base64,
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn generate_token() -> Result<String, JsError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)?;
    Ok(STANDARD.encode(bytes))
}

#[wasm_bindgen]
pub fn finalize(
    pub_key_pem: &str,
    blind_token_sign_base64: &str,
    blinding_result: JsValue,
    token_base64: &str,
) -> Result<String, JsError> {
    let pub_key = PublicKey::from_pem(pub_key_pem)?;
    let blind_token_sign = BlindSignature::new(STANDARD.decode(blind_token_sign_base64)?);
    let blinding_result = serde_wasm_bindgen::from_value::<BlindingResult>(blinding_result)?;
    let blinding_result = blind_rsa_signatures::BlindingResult {
        blind_message: BlindMessage::new(STANDARD.decode(&blinding_result.blind_token)?),
        secret: Secret::new(STANDARD.decode(&blinding_result.secret)?),
        msg_randomizer: None,
    };
    let token = STANDARD.decode(token_base64)?;

    let token_sign = pub_key.finalize(&blind_token_sign, &blinding_result, token)?;

    Ok(STANDARD.encode(token_sign))
}

#[wasm_bindgen]
pub fn verify(
    pub_key_pem: &str,
    token_sign_base64: &str,
    token_base64: &str,
) -> Result<bool, JsError> {
    let pub_key = PublicKey::from_pem(pub_key_pem)?;
    let token_sign = Signature::new(STANDARD.decode(token_sign_base64)?);
    match pub_key.verify(&token_sign, None, STANDARD.decode(token_base64)?) {
        Ok(_) => Ok(true),
        Err(_e) => Ok(false),
    }
}
