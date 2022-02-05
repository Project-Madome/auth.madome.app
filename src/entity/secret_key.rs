use std::ops::Deref;

use ring::{digest, rand::SystemRandom};

use super::token;

pub const SECRET_KEY_EXP: i64 = token::REFRESH_TOKEN_EXP;

#[cfg_attr(test, derive(Default))]
#[derive(Clone)]
pub struct SecretKey(pub String);

impl SecretKey {
    pub fn new() -> Self {
        // TODO: 테스트 할 일이 생기면 여기에서 생성하는 방법 말고 다른 곳에서 생성할 수 있는 방법을 찾아보자
        // 테스트에 값 주입은 생명이니까
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
