use std::sync::Arc;

use serde::Deserialize;

use crate::{
    error::{RepositoryError, UseCaseError},
    repository::{r#trait::AuthcodeRepository, RepositorySet},
};

#[derive(Deserialize, Clone)]
pub struct Payload {
    pub code: String,
}

pub struct Model {
    pub user_email: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    InvalidAuthcode,
    #[error("")]
    Repository(#[from] RepositoryError),
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload { code }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let maybe_authcode = repository.authcode().pop(&code).await?;

    match maybe_authcode {
        Some(authcode) => Ok(Model {
            user_email: authcode.user_email,
        }),
        _ => Err(Error::InvalidAuthcode.into()),
    }
}
