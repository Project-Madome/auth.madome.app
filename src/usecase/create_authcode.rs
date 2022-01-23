use std::sync::Arc;

use hyper::{Body, Response, StatusCode};
use serde::Deserialize;

use crate::{
    command::CommandSet,
    entity::authcode::Authcode,
    error::UseCaseError,
    repository::{r#trait::AuthcodeRepository, RepositorySet},
};

#[derive(Deserialize)]
pub struct Payload {
    #[serde(rename = "email")]
    pub user_email: String,
}

pub struct Model;

impl From<Model> for Response<Body> {
    fn from(_: Model) -> Self {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
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
    Payload { user_email }: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let code = command.random_code().await?;

    let authcode = Authcode {
        user_email: user_email.clone(),
        code: code.clone(),
    };

    let _ = repository.authcode().add(authcode).await?;

    command.send_email(user_email, code).await?;

    Ok(Model)
}
