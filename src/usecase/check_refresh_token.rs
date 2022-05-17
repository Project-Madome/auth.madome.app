use std::sync::Arc;

use util::ori;
use uuid::Uuid;

use crate::{
    entity::{secret_key::SecretKey, token::RefreshToken},
    error::UseCaseError,
    repository::{r#trait::SecretKeyRepository, RepositorySet},
};

pub struct Payload {
    pub refresh_token: String,
}

#[derive(Debug)]
pub struct Model {
    pub token_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unauthorized")]
    UnauthorizedRefreshToken,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

async fn deserialize(
    refresh_token: &str,
    secret_key_repository: Arc<impl SecretKeyRepository>,
) -> crate::Result<Option<RefreshToken>> {
    let token_id = ori!(RefreshToken::deserialize_payload(refresh_token)).id;

    let SecretKey(secret_key) = ori!(secret_key_repository.get(token_id).await?);

    let token_data = ori!(RefreshToken::deserialize(refresh_token, &secret_key)).claims;

    Ok(Some(token_data))
}

pub async fn execute(
    Payload { refresh_token }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let token_data = match deserialize(&refresh_token, repository.secret_key()).await? {
        Some(r) => r,
        None => return Err(Error::UnauthorizedRefreshToken.into()),
    };

    Ok(Model {
        token_id: token_data.id,
        user_id: token_data.user_id,
    })
}

#[cfg(test)]
mod tests {
    use sai::{Component, System};
    use util::{assert_debug, test_registry};
    use uuid::Uuid;

    use crate::{
        entity::token::Token,
        repository::{r#trait::SecretKeyRepository, RepositorySet},
        usecase::check_refresh_token::{self, Payload},
    };

    #[tokio::test]
    async fn success() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet] ->
        [secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret54231".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (_, serialized) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                refresh_token: serialized,
            };
            let r = check_refresh_token::execute(payload, repository)
                .await
                .unwrap();


            assert_eq!(r.token_id, token.id);
            assert_eq!(user_id, token.user_id);
        });
    }

    // check_access_token::deserialize 함수에서 None을 리턴하면 Unauthorized 에러가 난다는 것만 테스트하면 됨
    #[tokio::test]
    async fn error_unauthorized_by_use_access_token_instead_of_refresh_token() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet] ->
        [secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret54231".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (access_token, _) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                refresh_token: access_token,
            };
            let r = check_refresh_token::execute(payload, repository)
                .await
                .expect_err("expected error, but returns ok");


            assert_debug!(r, crate::Error::from(check_refresh_token::Error::UnauthorizedRefreshToken));
        });
    }
}
