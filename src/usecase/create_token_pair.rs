use std::sync::Arc;

use crate::{
    command::CommandSet,
    entity::{secret_key::SecretKey, token::Token},
    error::UseCaseError,
    model::TokenPair,
    repository::{r#trait::SecretKeyRepository, RepositorySet},
};

use super::check_authcode;

pub enum Payload {
    UserEmail(String),
    UserId(String),
}

impl From<check_authcode::Model> for Payload {
    fn from(model: check_authcode::Model) -> Self {
        Self::UserEmail(model.user_email)
    }
}

/* impl From<check_token_pair::Model> for Payload {
    fn from(model: check_token_pair::Model) -> Self {
        Self::UserId(model.user_id)
    }
} */

pub type Model = TokenPair;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found user")]
    NotFoundUser,

    #[error("Can't added secret key")]
    CannotAddedSecretKey,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    payload: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let user_id = match payload {
        Payload::UserId(user_id) => user_id,
        Payload::UserEmail(user_email) => match command.get_user_info(user_email).await? {
            Some(user) => user.id,
            None => return Err(Error::NotFoundUser.into()),
        },
    };

    let token = Token::new(user_id);
    let secret_key = SecretKey::new();

    let secret_key_added = repository.secret_key().add(&token.id, &secret_key).await?;

    if !secret_key_added {
        return Err(Error::CannotAddedSecretKey.into());
    }

    let (access_token, refresh_token) = token.serialize(&secret_key)?;

    Ok(Model {
        access_token,
        refresh_token,
    })
}

#[cfg(test)]
mod tests {
    use sai::{Component, System};
    use util::test_registry;
    use uuid::Uuid;

    use crate::{
        command::CommandSet,
        repository::RepositorySet,
        usecase::{check_token_pair, create_token_pair},
    };

    #[tokio::test]
    async fn success() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [user_id: String] ->
        {
            user_id = Uuid::new_v4().to_string();
        },
        {
            let payload = create_token_pair::Payload::UserId(user_id.clone());
            let r = create_token_pair::execute(payload, repository.clone(), command.clone()).await.unwrap();

            let payload = check_token_pair::Payload {
                access_token: r.access_token,
                refresh_token: r.refresh_token
            };
            let r = check_token_pair::execute(payload, repository, command).await.unwrap();

            assert_eq!(r.user_id, user_id);
        });
    }
}
