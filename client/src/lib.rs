use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use blind_rsa_signatures::{
    BlindMessage, BlindSignature, DefaultRng, MessageRandomizer, PSS, Randomized, Secret, Sha384,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

type PublicKey = blind_rsa_signatures::PublicKey<Sha384, PSS, Randomized>;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlindingResult {
    pub blind_token: String,
    pub secret: String,
    pub msg_randomizer: String,
}

#[wasm_bindgen]
pub fn blind(token_base64: &str, pub_key_pem: &str) -> Result<JsValue, JsError> {
    let token_bytes = STANDARD.decode(token_base64)?;
    let pub_key: PublicKey = PublicKey::from_pem(pub_key_pem)?;
    let blind_token = pub_key.blind(&mut DefaultRng, token_bytes)?;
    let blind_token_base64 = STANDARD.encode(blind_token.blind_message);
    let secret_base64 = STANDARD.encode(blind_token.secret);
    let randomizer_base64 = STANDARD.encode(blind_token.msg_randomizer.unwrap());
    let result = BlindingResult {
        blind_token: blind_token_base64,
        secret: secret_base64,
        msg_randomizer: randomizer_base64,
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
    let pub_key: PublicKey = PublicKey::from_pem(pub_key_pem)?;
    let blind_token_sign = BlindSignature::new(STANDARD.decode(blind_token_sign_base64)?);
    let blinding_result = serde_wasm_bindgen::from_value::<BlindingResult>(blinding_result)?;
    let blinding_result = blind_rsa_signatures::BlindingResult {
        blind_message: BlindMessage::new(STANDARD.decode(&blinding_result.blind_token)?),
        secret: Secret::new(STANDARD.decode(&blinding_result.secret)?),
        msg_randomizer: Some(MessageRandomizer::new(
            STANDARD
                .decode(&blinding_result.msg_randomizer)?
                .try_into()
                .map_err(|_| JsError::new("Invalid message randomizer"))?,
        )),
    };
    let token = STANDARD.decode(token_base64)?;

    let token_sign = pub_key.finalize(&blind_token_sign, &blinding_result, token)?;

    Ok(STANDARD.encode(token_sign))
}
