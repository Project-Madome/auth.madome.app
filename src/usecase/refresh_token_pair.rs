use std::sync::Arc;

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
mod tests {
    // TODO: success, error_invalid_auth_code(시간 초과된 걸로 테스트 해야함, 시간을 설정할 수 있게 가능한가?)
}
