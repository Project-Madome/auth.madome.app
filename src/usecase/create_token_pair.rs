use std::sync::Arc;

use hyper::{Body, Response, StatusCode};
use serde::Serialize;

use crate::{
    command::CommandSet,
    constant::http::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN},
    entity::token::Token,
    error::UseCaseError,
    utils::http::{SetCookie, SetCookieOptions, SetHeaders},
};

use super::{check_authcode, check_token_pair};

pub struct Payload {
    pub user_email: Option<String>,
    pub user_id: Option<String>,
}

impl From<check_authcode::Model> for Payload {
    fn from(model: check_authcode::Model) -> Self {
        Self {
            user_email: Some(model.user_email),
            user_id: None,
        }
    }
}

impl From<check_token_pair::Model> for Payload {
    fn from(model: check_token_pair::Model) -> Self {
        Self {
            user_email: None,
            user_id: Some(model.user_id),
        }
    }
}

#[derive(Serialize)]
pub struct Model {
    pub access_token: String,
    pub refresh_token: String,
}

impl From<Model> for Response<Body> {
    fn from(
        Model {
            access_token,
            refresh_token,
        }: Model,
    ) -> Self {
        let set_cookie_options = SetCookieOptions::new().domain("madome.app").http_only(true);
        let set_cookie = SetCookie::new()
            .set(
                MADOME_ACCESS_TOKEN,
                access_token,
                set_cookie_options.clone().max_age(3600 * 4),
            )
            .set(
                MADOME_REFRESH_TOKEN,
                refresh_token,
                set_cookie_options.max_age(3600 * 24 * 7),
            );

        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .headers(set_cookie.iter())
            .body(Body::empty())
            // .header(header::CONTENT_TYPE, header::APPLICATION_JSON)
            // .body(Body::from(serialized))
            .unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload {
        user_email,
        user_id,
    }: Payload,
    command: Arc<CommandSet>,
    secret_key: &str,
) -> crate::Result<Model> {
    let user_id = match (user_id, user_email) {
        (Some(user_id), _) => user_id,
        (_, Some(user_email)) => command.get_user_info(user_email).await?.id,
        _ => unreachable!(),
    };

    let (access_token, refresh_token) = Token::new(user_id).serialize(secret_key)?;

    Ok(Model {
        access_token,
        refresh_token,
    })
}
