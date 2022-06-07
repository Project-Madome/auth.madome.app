use std::sync::Arc;

use hyper::{Body, Request};
use madome_sdk::api::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN};
use util::http::Cookie;
use uuid::Uuid;

use crate::{
    command::CommandSet,
    error::UseCaseError,
    repository::{r#trait::SecretKeyRepository, RepositorySet},
};

use super::{check_token_pair, create_token_pair};

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

pub struct Model {
    pub access_token: String,
    pub refresh_token: String,
    pub token_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Can't removed secret key")]
    CannotRemovedSecretKey,
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
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let token_data = check_token_pair::execute(
        (access_token, refresh_token).into(),
        repository.clone(),
        command.clone(),
    )
    .await?;

    // remove secretkey of prev token
    let secret_key_removed = repository.secret_key().remove(token_data.token_id).await?;

    if !secret_key_removed {
        return Err(Error::CannotRemovedSecretKey.into());
    }

    let t = create_token_pair::execute(
        create_token_pair::Payload::UserId(token_data.user_id),
        repository.clone(),
        command.clone(),
    )
    .await?;

    Ok(Model {
        access_token: t.access_token,
        refresh_token: t.refresh_token,
        token_id: t.token_id,
        user_id: t.user_id,
    })
}

#[cfg(test)]
mod tests {}
