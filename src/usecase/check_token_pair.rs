use std::{convert::TryFrom, sync::Arc};

use hyper::{Body, Request};
use madome_sdk::api::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN};
use serde::Deserialize;
use util::http::Cookie;
use uuid::Uuid;

use crate::{command::CommandSet, error::UseCaseError, repository::RepositorySet};

use super::{check_access_token, check_refresh_token};

#[derive(Deserialize)]
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

impl From<(String, String)> for Payload {
    fn from((access_token, refresh_token): (String, String)) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug)]
pub struct Model {
    pub user_id: Uuid,
    pub token_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid token pair")]
    InvalidTokenPair,
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
    let access_token = check_access_token::execute(
        check_access_token::Payload {
            access_token,
            minimum_role: None,
            validate_exp: false,
        },
        repository.clone(),
        command,
    )
    .await?;

    let refresh_token =
        check_refresh_token::execute(check_refresh_token::Payload { refresh_token }, repository)
            .await?;

    if access_token.token_id != refresh_token.token_id
        || access_token.user_id != refresh_token.user_id
    {
        return Err(Error::InvalidTokenPair.into());
    }

    Ok(Model {
        user_id: access_token.user_id,
        token_id: access_token.token_id,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sai::{Component, System};
    use util::{assert_debug, test_registry};
    use uuid::Uuid;

    use crate::{
        command::CommandSet,
        entity::token::Token,
        repository::{r#trait::SecretKeyRepository, RepositorySet},
        usecase::check_token_pair::{self, Payload},
    };

    #[tokio::test]
    async fn success() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret0382".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (access_token, refresh_token) = token.serialize(&secret_key).expect("serialize jwt");

            let payload = Payload {
                access_token,
                refresh_token,
            };

            let r = check_token_pair::execute(payload, repository, command)
                .await
                .unwrap();

            assert_eq!(r.user_id, user_id);
        });
    }

    #[tokio::test]
    async fn error_unauthorized_by_not_same_token_id() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: Uuid, a_token: Token, b_token: Token] ->
        {
            secret_key = "secret03223".to_string();
            user_id = Uuid::new_v4();
            a_token = Token::new(user_id);
            b_token = Token::new(user_id);

            a_token.id = Uuid::from_str("96e220fe-cb9b-40f5-9f88-0c023a349b59").unwrap();
            b_token.id = Uuid::from_str("d344c1db-8a6d-42d1-bc2d-d488ab8d46b6").unwrap();

            repository
                .secret_key()
                .add(a_token.id, &secret_key)
                .await
                .unwrap();

            repository
                .secret_key()
                .add(b_token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (access_token, _) = a_token.serialize(&secret_key).expect("serialize jwt");
            let (_, refresh_token) = b_token.serialize(&secret_key).expect("serialize jwt");

            let payload = Payload {
                access_token,
                refresh_token,
            };

            let r = check_token_pair::execute(payload, repository, command)
                .await
                .expect_err("expected error, but returns ok");

            assert_debug!(r, crate::Error::from(check_token_pair::Error::InvalidTokenPair));
        });
    }

    #[tokio::test]
    async fn error_unauthorized_by_not_same_user_id() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, a_user_id: Uuid, b_user_id: Uuid, a_token: Token, b_token: Token] ->
        {
            secret_key = "secret03458".to_string();
            a_user_id = Uuid::from_str("bf4cf9fe-961f-4aba-a11b-17cf43c7ed39").unwrap();
            b_user_id = Uuid::from_str("7a129706-4458-493c-a0b9-11f5a57fffa7").unwrap();
            a_token = Token::new(a_user_id);
            b_token = Token::new(b_user_id);

            b_token.id = a_token.id;

            // println!("{:?}", a_token);
            // println!("{:?}", b_token);

            repository
                .secret_key()
                .add(a_token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (access_token, _) = a_token.serialize(&secret_key).expect("serialize jwt");
            let (_, refresh_token) = b_token.serialize(&secret_key).expect("serialize jwt");

            let payload = Payload {
                access_token,
                refresh_token,
            };

            let r = check_token_pair::execute(payload, repository, command)
                .await
                .expect_err("expected error, but returns ok");

            assert_debug!(r, crate::Error::from(check_token_pair::Error::InvalidTokenPair));
        });
    }
}
