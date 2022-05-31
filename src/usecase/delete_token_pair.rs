use std::sync::Arc;

use hyper::{Body, Request};
use madome_sdk::api::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN};
use util::http::Cookie;

use crate::{
    entity::token::{AccessToken, RefreshToken},
    error::UseCaseError,
    repository::{r#trait::SecretKeyRepository, RepositorySet},
};

#[derive(Debug)]
pub struct Payload {
    pub access_token: String,
    pub refresh_token: String,
}

impl TryFrom<Request<Body>> for Payload {
    type Error = crate::Error;

    fn try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let mut cookie = Cookie::from(&request);

        let access_token = cookie.take(MADOME_ACCESS_TOKEN).unwrap_or_default();
        let refresh_token = cookie.take(MADOME_REFRESH_TOKEN).unwrap_or_default();

        Ok(Self {
            access_token,
            refresh_token,
        })
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid token")]
    InvalidToken,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload {
        access_token,
        refresh_token,
    }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    // TODO:
    // 지우기만 하면 되기 때문에 유효한지 검증할 필요가 없음

    let a = AccessToken::deserialize_payload(&access_token);
    let r = RefreshToken::deserialize_payload(&refresh_token);

    let token_id = match (a, r) {
        (Some(a), Some(r)) if a.id == r.id && a.user_id == r.user_id => a.id,
        (Some(a), None) => a.id,
        (_, Some(r)) => r.id,
        _ => return Err(Error::InvalidToken.into()),
    };

    // 에러만 안나면 됨
    let _r = repository.secret_key().remove(token_id).await?;

    Ok(Model)
}
