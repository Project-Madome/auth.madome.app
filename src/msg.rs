use std::convert::TryInto;

use hyper::{http::response::Builder as ResponseBuilder, Body, Method, Request};
use serde::de::DeserializeOwned;

use util::{r#async::AsyncTryFrom, IntoPayload, ReadChunks};

use crate::usecase::{
    check_access_token, check_authcode, create_authcode, delete_token_pair, refresh_token_pair,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Json deserialize: {0}")]
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
    // RefreshTokenPair(check_token_pair::Payload),
    CheckAccessToken(check_access_token::Payload),
    RefreshTokenPair(refresh_token_pair::Payload),
    DeleteTokenPair(delete_token_pair::Payload),
}

impl Msg {
    pub async fn from_http(
        request: Request<Body>,
        response: ResponseBuilder,
    ) -> crate::Result<(Self, ResponseBuilder)> {
        let method = request.method().clone();
        let path = request.uri().path();

        // log::debug!("request headers = {:?}", request.headers());

        // cfg(feature = "production")
        // TODO: 이걸 써야하는 곳을 잘 생각해 인증쪽에서

        let msg = match (method, path) {
            (Method::GET, "/auth/token") => Msg::CheckAccessToken(request.try_into()?),
            (Method::POST, "/auth/token") => {
                Msg::CreateTokenPair(Wrap::async_try_from(request).await?.inner())
            }
            (Method::PATCH, "/auth/token") => Msg::RefreshTokenPair(request.try_into()?),
            (Method::DELETE, "/auth/token") => Msg::DeleteTokenPair(request.try_into()?),
            (Method::POST, "/auth/code") => Msg::CreateAuthcode(request.into_payload(()).await?),

            _ => return Err(Error::NotFound.into()),
        };

        Ok((msg, response))
    }
}

pub struct Wrap<P>(pub P);

impl<P> Wrap<P> {
    pub fn inner(self) -> P {
        self.0
    }
}

#[async_trait::async_trait]
impl<P> AsyncTryFrom<Request<Body>> for Wrap<P>
where
    P: DeserializeOwned,
{
    type Error = crate::Error;

    async fn async_try_from(mut request: Request<Body>) -> Result<Self, Self::Error> {
        let chunks = request.body_mut().read_chunks().await?;

        let payload =
            serde_json::from_slice::<P>(&chunks).map_err(Error::JsonDeserializePayload)?;

        Ok(Wrap(payload))
    }
}
