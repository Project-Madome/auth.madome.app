use std::sync::Arc;

use hyper::{Body, Response, StatusCode};
use serde::Serialize;

use crate::{
    command::CommandSet, constant::http::header, entity::token::Token, error::UseCaseError,
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

/* #[async_trait::async_trait]
impl AsyncTryFrom<Request<Body>> for Payload {
    type Error = crate::Error;

    async fn try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let chunks = request.body_mut().read_chunks().await?;

        let payload =
            serde_json::from_slice::<Payload>(&chunks).map_err(crate::Error::JsonDeserialize);

        payload
    }
} */

#[derive(Serialize)]
pub struct Model {
    pub access_token: String,
    pub refresh_token: String,
}

impl From<Model> for Response<Body> {
    fn from(model: Model) -> Self {
        let serialized = serde_json::to_vec(&model).expect("json serialize");

        Response::builder()
            .status(StatusCode::OK)
            // TODO: Set-Cookie
            .header(header::CONTENT_TYPE, header::APPLICATION_JSON)
            .body(Body::from(serialized))
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
