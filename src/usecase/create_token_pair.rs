use std::sync::Arc;

use serde::Serialize;

use crate::{
    command::CommandSet,
    entity::{secret_key::SecretKey, token::Token},
    error::UseCaseError,
    repository::{r#trait::SecretKeyRepository, RepositorySet},
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

#[derive(Serialize, Debug)]
pub struct Model {
    pub access_token: String,
    pub refresh_token: String,
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
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let user_id = match (user_id, user_email) {
        (Some(user_id), _) => user_id,
        (_, Some(user_email)) => command.get_user_info(user_email).await?.id,
        _ => unreachable!(),
    };

    let token = Token::new(user_id);
    let secret_key = SecretKey::new();

    let _ = repository.secret_key().add(&token.id, &secret_key).await?;

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
            let payload = create_token_pair::Payload {
                user_id: Some(user_id.clone()),
                user_email: None
            };
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
