use std::sync::Arc;

use serde::Deserialize;

use crate::{
    error::UseCaseError,
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
    #[error("Invalid authcode")]
    InvalidAuthcode,
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
        Some(authcode) if !authcode.expired() => Ok(Model {
            user_email: authcode.user_email,
        }),
        _ => Err(Error::InvalidAuthcode.into()),
    }
}

#[cfg(test)]
mod tests {
    // TODO: success, error_invalid_auth_code(시간 초과된 걸로 테스트 해야함, 시간을 설정할 수 있게 가능한가?)
}
