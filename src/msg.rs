use std::convert::TryInto;

use hyper::{Body, Method, Request};
use serde::de::DeserializeOwned;

use crate::{
    usecase::{check_access_token, check_authcode, check_token_pair, create_authcode},
    utils::{
        r#async::{AsyncTryFrom, AsyncTryInto},
        ReadChunks,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    NotFound,
    #[error("")]
    JsonDeserializePayload(serde_json::Error),
}

/// Msg의 Payload는 같은 이름의 usecase의 Payload와는 관계가 없음
///
/// Msg의 Payload는 실행되어야하는 usecase 순서에 따라 정해짐 (제일 처음 실행하는 usecase의 Payload)
///
/// 실행되는 순서는 Resolver 참조
pub enum Msg {
    CreateAuthcode(create_authcode::Payload),
    CreateTokenPair(check_authcode::Payload),
    RefreshTokenPair(check_token_pair::Payload),
    CheckAccessToken(check_access_token::Payload),
}

#[async_trait::async_trait]
impl AsyncTryFrom<Request<Body>> for Msg {
    type Error = crate::Error;

    async fn async_try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let method = request.method().clone();
        let path = request.uri().path();

        let msg = match (method, path) {
            (Method::GET, "/auth/token") => Msg::CheckAccessToken(request.try_into()?),
            (Method::POST, "/auth/token") => Msg::CreateTokenPair(request.async_try_into().await?),
            (Method::PATCH, "/auth/token") => Msg::RefreshTokenPair(request.try_into()?),
            (Method::POST, "/auth/code") => Msg::CreateAuthcode(request.async_try_into().await?),
            _ => return Err(Error::NotFound.into()),
        };

        Ok(msg)
    }
}

#[async_trait::async_trait]
impl<P> AsyncTryFrom<Request<Body>> for P
where
    P: DeserializeOwned,
{
    type Error = crate::Error;

    async fn async_try_from(mut request: Request<Body>) -> Result<Self, Self::Error> {
        let chunks = request.body_mut().read_chunks().await?;

        let payload =
            serde_json::from_slice::<P>(&chunks).map_err(Error::JsonDeserializePayload)?;

        Ok(payload)
    }
}
