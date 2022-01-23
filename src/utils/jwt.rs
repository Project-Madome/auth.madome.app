use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::de::DeserializeOwned;
use serde::Serialize;

/*
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,         // Optional. Audience
    exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize,          // Optional. Issued at (as UTC timestamp)
    iss: String,         // Optional. Issuer
    nbf: usize,          // Optional. Not Before (as UTC timestamp)
    sub: String,         // Optional. Subject (whom token refers to)
}
*/

pub fn serialize<Claims: Serialize>(
    claims: &Claims,
    secret_key: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret_key.as_ref());

    jsonwebtoken::encode(&header, claims, &encoding_key)
}

pub fn deserialize<Claims: DeserializeOwned>(
    token: &str,
    secret_key: &str,
    validate_exp: bool,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
    let validation = Validation {
        validate_exp,
        ..Default::default()
    };

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
}
