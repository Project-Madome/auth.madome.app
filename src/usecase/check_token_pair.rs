use std::{convert::TryFrom, sync::Arc};

use hyper::{Body, Request};
use serde::Deserialize;

use crate::{
    command::CommandSet,
    constant::http::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN},
    error::UseCaseError,
    utils::http::Cookie,
};

use super::{check_access_token, check_refresh_token};

#[derive(Deserialize)]
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

pub struct Model {
    pub user_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    InvalidTokenPair,
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
    command: Arc<CommandSet>,
    secret_key: &str,
) -> crate::Result<Model> {
    let access_token = check_access_token::execute(
        check_access_token::Payload {
            access_token,
            minimum_role: None,
            validate_exp: false,
        },
        command,
        secret_key,
    )
    .await?
    .token;

    let refresh_token =
        check_refresh_token::execute(check_refresh_token::Payload { refresh_token }, secret_key)
            .await?
            .token;

    if access_token.id != refresh_token.id || access_token.user_id != refresh_token.user_id {
        return Err(Error::InvalidTokenPair.into());
    }

    Ok(Model {
        user_id: access_token.user_id,
    })
}
