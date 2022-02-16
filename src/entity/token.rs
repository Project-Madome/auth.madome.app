use chrono::Utc;
use jsonwebtoken::TokenData;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

pub const ACCESS_TOKEN_EXP: i64 = 3600 * 4;
pub const REFRESH_TOKEN_EXP: i64 = 3600 * 24 * 7;

pub mod jwt {
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
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
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = validate_exp;

        jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
    }
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Clone)]
pub struct Token {
    pub id: String,
    pub user_id: String,
}

#[cfg_attr(test, derive(Default, Clone))]
#[derive(Serialize, Deserialize)]
pub struct AccessToken {
    pub sub: String,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,

    pub id: String,
    pub user_id: String,

    /// placeholder for access token
    ///
    /// serialize할 때 이게 있으면 access_token이라는 증거
    pub _a: bool,
}

impl AccessToken {
    pub fn deserialize(
        access_token: &str,
        secret_key: &str,
        validate_exp: bool,
    ) -> Option<TokenData<Self>> {
        Token::deserialize(access_token, secret_key, validate_exp)
    }

    pub fn deserialize_payload(access_token: &str) -> Option<Self> {
        Token::deserialize_payload(access_token)
    }
}

impl From<Token> for AccessToken {
    fn from(Token { id, user_id }: Token) -> Self {
        let issued_at = Utc::now().timestamp();

        Self {
            sub: "madome access token".to_string(),
            iss: "madome.app".to_string(),
            iat: issued_at,
            exp: issued_at + ACCESS_TOKEN_EXP,
            id,
            user_id,
            _a: true,
        }
    }
}

#[cfg_attr(test, derive(Default, Clone))]
#[derive(Serialize, Deserialize)]
pub struct RefreshToken {
    pub sub: String,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,

    pub id: String,
    pub user_id: String,

    /// placeholder for refresh token
    ///
    /// serialize할 때 이게 있으면 refresh_token이라는 증거
    pub _r: bool,
}

impl RefreshToken {
    pub fn deserialize(refresh_token: &str, secret_key: &str) -> Option<TokenData<Self>> {
        Token::deserialize(refresh_token, secret_key, true)
    }

    pub fn deserialize_payload(refresh_token: &str) -> Option<Self> {
        Token::deserialize_payload(refresh_token)
    }
}

impl From<Token> for RefreshToken {
    fn from(Token { id, user_id }: Token) -> Self {
        let issued_at = Utc::now().timestamp();

        Self {
            sub: "madome refresh token".to_string(),
            iss: "madome.app".to_string(),
            iat: issued_at,
            exp: issued_at + REFRESH_TOKEN_EXP,
            id,
            user_id,
            _r: true,
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

    pub fn deserialize<T>(token: &str, secret_key: &str, validate_exp: bool) -> Option<TokenData<T>>
    where
        T: DeserializeOwned,
    {
        jwt::deserialize::<T>(token, secret_key, validate_exp).ok()
    }

    pub fn deserialize_payload<T>(token: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        token.split('.').nth(1).and_then(|st| {
            base64::decode_config(st, base64::URL_SAFE_NO_PAD)
                .ok()
                .and_then(|p| serde_json::from_slice(&p).ok())
        })
    }
}
