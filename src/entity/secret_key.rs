use std::ops::Deref;

use ring::{digest, rand::SystemRandom};

use super::token;

pub const SECRET_KEY_EXP: i64 = token::REFRESH_TOKEN_EXP;

#[derive(Clone, Default)]
pub struct SecretKey(pub String);

impl SecretKey {
    pub fn new() -> Self {
        let rng = SystemRandom::new();

        let random_bytes = ring::rand::generate::<[u8; 256]>(&rng).unwrap().expose();

        let hashed = digest::digest(&digest::SHA512, &random_bytes);

        let key = base64::encode_config(hashed, base64::BINHEX);

        Self(key)
    }
}

impl Deref for SecretKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
