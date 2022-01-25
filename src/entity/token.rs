use chrono::Utc;
use jsonwebtoken::TokenData;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::jwt;

#[derive(Clone)]
pub struct Token {
    pub id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccessToken {
    pub sub: String,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,

    pub id: String,
    pub user_id: String,
}

impl AccessToken {
    pub fn deserialize(
        access_token: &str,
        secret_key: &str,
        validate_exp: bool,
    ) -> Option<TokenData<Self>> {
        Token::deserialize(access_token, secret_key, validate_exp)
    }
}

impl From<Token> for AccessToken {
    fn from(Token { id, user_id }: Token) -> Self {
        let issued_at = Utc::now().timestamp();

        Self {
            sub: "madome access token".to_string(),
            iss: "madome.app".to_string(),
            iat: issued_at,
            exp: issued_at + 3600 * 4,
            id,
            user_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshToken {
    pub sub: String,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,

    pub id: String,
    pub user_id: String,
}

impl RefreshToken {
    pub fn deserialize(refresh_token: &str, secret_key: &str) -> Option<TokenData<Self>> {
        Token::deserialize(refresh_token, secret_key, true)
    }
}

impl From<Token> for RefreshToken {
    fn from(Token { id, user_id }: Token) -> Self {
        let issued_at = 0;

        Self {
            sub: "madome refresh token".to_string(),
            iss: "madome.app".to_string(),
            iat: issued_at,
            exp: issued_at + 3600 * 24 * 7,
            id,
            user_id,
        }
    }
}

impl Token {
    pub fn new(user_id: String) -> Self {
        let id = Uuid::new_v4().to_string();

        Self { id, user_id }
    }

    /// # Return
    /// (AccessToken, RefreshToken)
    pub fn serialize(&self, secret_key: &str) -> crate::Result<(String, String)> {
        let access_token = jwt::serialize(&AccessToken::from(self.clone()), secret_key)
            .expect("jsonwebtoken serialize");
        let refresh_token = jwt::serialize(&RefreshToken::from(self.clone()), secret_key)
            .expect("jsonwebtoken serialize");

        Ok((access_token, refresh_token))
    }

    pub fn deserialize<T: DeserializeOwned>(
        token: &str,
        secret_key: &str,
        validate_exp: bool,
    ) -> Option<TokenData<T>> {
        jwt::deserialize::<T>(token, secret_key, validate_exp).ok()
    }
}
