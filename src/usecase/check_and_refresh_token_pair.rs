use std::{collections::HashMap, convert::TryFrom, sync::Arc};

use hyper::{Body, Request};
use madome_sdk::api::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN};
use serde::Serialize;
use util::http::Cookie;
use uuid::Uuid;

use crate::{
    command::CommandSet, error::UseCaseError, model::TokenPair, repository::RepositorySet,
};

use super::{check_access_token, refresh_token_pair};

pub struct Payload {
    pub access_token: String,
    pub refresh_token: String,
    pub minimum_role: Option<u8>,
}

impl TryFrom<Request<Body>> for Payload {
    type Error = crate::Error;

    fn try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let mut cookie = Cookie::from(&request);
        let qs = querystring::querify(request.uri().query().unwrap_or(""))
            .into_iter()
            .collect::<HashMap<_, _>>();

        let access_token = cookie.take(MADOME_ACCESS_TOKEN).unwrap_or_default();
        let refresh_token = cookie.take(MADOME_REFRESH_TOKEN).unwrap_or_default();
        let minimum_role = qs.get("role").and_then(|v| v.parse().ok());

        Ok(Self {
            access_token,
            refresh_token,
            minimum_role,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Model {
    #[serde(skip_serializing)]
    pub access_token: Option<String>,
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing)]
    pub token_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Permission denied")]
    PermissionDenied(TokenPair),
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

/// # Return
/// - Unauthorized -> None
/// - Other -> Some(err)
fn is_not_unauthorized(err: crate::Error) -> Option<crate::Error> {
    use crate::error::UseCaseError::*;
    use crate::usecase::check_access_token::Error::*;
    use crate::Error::*;

    match err {
        UseCase(CheckAccessToken(UnauthorizedAccessToken)) => None,
        err => Some(err),
    }
}

/// # Return
/// - PermissionDenied -> None
/// - Other -> Some(err)
fn is_not_permission_denied(err: crate::Error) -> Option<crate::Error> {
    use crate::error::UseCaseError::*;
    use crate::usecase::check_access_token::Error::*;
    use crate::Error::*;

    match err {
        UseCase(CheckAccessToken(PermissionDenied)) => None,
        err => Some(err),
    }
}

pub async fn execute(
    Payload {
        access_token,
        refresh_token,
        minimum_role,
    }: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let r_check_access_token = check_access_token::execute(
        check_access_token::Payload {
            access_token: access_token.clone(),
            minimum_role,
            validate_exp: true,
        },
        repository.clone(),
        command.clone(),
    )
    .await
    .map_err(is_not_unauthorized);

    // match Err::<check_access_token::Model, _>(None) {
    match r_check_access_token {
        // PermissionDenied ?????? ?????? ??????
        Err(Some(err)) => Err(err),
        // ??????
        Ok(r) => {
            let r = Model {
                access_token: None,
                refresh_token: None,
                token_id: r.token_id,
                user_id: r.user_id,
            };

            Ok(r)
        }
        Err(None) => {
            let t = refresh_token_pair::execute(
                refresh_token_pair::Payload {
                    access_token,
                    refresh_token,
                },
                repository.clone(),
                command.clone(),
            )
            .await?;

            let r = if let Some(minimum_role) = minimum_role {
                let r = check_access_token::execute(
                    check_access_token::Payload {
                        access_token: t.access_token.clone(),
                        minimum_role: Some(minimum_role),
                        validate_exp: true,
                    },
                    repository,
                    command,
                )
                .await
                .map_err(is_not_permission_denied);

                Some(r)
            } else {
                None
            };

            match r {
                None | Some(Ok(_)) => Ok(Model {
                    access_token: Some(t.access_token),
                    refresh_token: Some(t.refresh_token),
                    token_id: t.token_id,
                    user_id: t.user_id,
                }),
                Some(Err(None)) => Err(Error::PermissionDenied(TokenPair {
                    access_token: t.access_token,
                    refresh_token: t.refresh_token,
                })
                .into()),
                Some(Err(Some(err))) => Err(err),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use madome_sdk::api::user::model::User;
    use sai::{Component, System};
    use util::{assert_debug, test_registry};
    use uuid::Uuid;

    use crate::command::{self, CommandSet};
    use crate::entity::token::{
        self, AccessToken, RefreshToken, Token, ACCESS_TOKEN_EXP, REFRESH_TOKEN_EXP,
    };
    use crate::model::TokenPair;
    use crate::repository::{r#trait::SecretKeyRepository, RepositorySet};
    use crate::usecase::check_access_token;
    use crate::usecase::check_and_refresh_token_pair::{self, Payload};

    #[tokio::test]
    async fn success_without_refresh() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (access_token, refresh_token) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                access_token,
                refresh_token,
                minimum_role: None,
            };
            let r = check_and_refresh_token_pair::execute(payload, repository, command)
                .await
                .unwrap();

            assert!(r.access_token.is_none());
            assert!(r.refresh_token.is_none());
            assert_eq!(r.token_id, token.id);
            assert_eq!(r.user_id, user_id);
        });
    }

    #[tokio::test]
    async fn success_with_refresh() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [access_token: AccessToken, refresh_token: RefreshToken, secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            let now = Utc::now().timestamp();

            access_token = AccessToken {
                sub: "madome access token".to_string(),
                iss: "madome.app".to_string(),
                iat: now,
                exp: now - ACCESS_TOKEN_EXP - 30,
                id: token.id,
                user_id,
                _a: true,
            };
            refresh_token = RefreshToken {
                sub: "madome refresh token".to_string(),
                iss: "madome.app".to_string(),
                iat: now,
                exp: now + REFRESH_TOKEN_EXP,
                id: token.id,
                user_id,
                _r: true,
            };

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let access_token = token::jwt::serialize(&access_token, &secret_key)
                .expect("access_token serialize");

            let refresh_token = token::jwt::serialize(&refresh_token, &secret_key)
                .expect("refresh_token serialize");

            let payload = Payload {
                access_token,
                refresh_token,
                minimum_role: None,
            };
            let r = check_and_refresh_token_pair::execute(payload, repository.clone(), command)
                .await
                .expect("expected ok, but returns error");

            let access_token = r.access_token.unwrap();
            let refresh_token = r.refresh_token.unwrap();

            let p = AccessToken::deserialize_payload(&access_token).expect("deserialize payload from access token");

            let secret_key = repository
                .secret_key()
                .get(p.id)
                .await
                .unwrap()
                .unwrap();

            AccessToken::deserialize(&access_token, &secret_key, true).expect("deserialize access token");
            RefreshToken::deserialize(&refresh_token, &secret_key).expect("deserialize refresh token");
        });
    }

    #[tokio::test]
    async fn error_permission_denied_without_refresh() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();

            let get_user_info = command::tests::GetUser::from(User {
                id: user_id,
                email: "".to_string(),
                role: 0,
                name: "".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now()
            });

            command.set_get_user_info(get_user_info);
        },
        {
            let (access_token, refresh_token) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                access_token,
                refresh_token,
                minimum_role: Some(1),
            };
            let r = check_and_refresh_token_pair::execute(payload, repository, command)
                .await
                .expect_err("expected error, but returns ok");

            let expected: crate::Error = check_access_token::Error::PermissionDenied.into();

            assert_debug!(r, expected);
        });
    }

    #[tokio::test]
    async fn error_permission_denied_with_refresh() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [access_token: AccessToken, refresh_token: RefreshToken, secret_key: String, user_id: Uuid, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4();
            token = Token::new(user_id);

            let now = Utc::now().timestamp();

            access_token = AccessToken {
                sub: "madome access token".to_string(),
                iss: "madome.app".to_string(),
                iat: now,
                exp: now - ACCESS_TOKEN_EXP - 30,
                id: token.id,
                user_id,
                _a: true,
            };
            refresh_token = RefreshToken {
                sub: "madome refresh token".to_string(),
                iss: "madome.app".to_string(),
                iat: now,
                exp: now + REFRESH_TOKEN_EXP,
                id: token.id,
                user_id,
                _r: true,
            };

            repository
                .secret_key()
                .add(token.id, &secret_key)
                .await
                .unwrap();

            let get_user_info = command::tests::GetUser::from(User {
                id: user_id,
                email: "".to_string(),
                role: 0,
                name: "".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now()
            });

            command.set_get_user_info(get_user_info);
        },
        {
            let access_token = token::jwt::serialize(&access_token, &secret_key)
                .expect("access_token serialize");

            let refresh_token = token::jwt::serialize(&refresh_token, &secret_key)
                .expect("refresh_token serialize");

            let payload = Payload {
                access_token,
                refresh_token,
                minimum_role: Some(1),
            };
            let r = check_and_refresh_token_pair::execute(payload, repository.clone(), command)
                .await
                .expect_err("expected error, but returns ok");

            {
                use crate::Error::UseCase;
                use crate::error::UseCaseError::CheckAndRefreshTokenPair;
                use check_and_refresh_token_pair::Error::*;

                match r {
                    UseCase(CheckAndRefreshTokenPair(PermissionDenied(TokenPair { access_token, refresh_token }))) => {
                        let p = AccessToken::deserialize_payload(&access_token).expect("deserialize payload from access token");

                        let secret_key = repository.secret_key().get(p.id).await.unwrap().unwrap();

                        AccessToken::deserialize(&access_token, &secret_key, true).expect("deserialize access token");
                        RefreshToken::deserialize(&refresh_token, &secret_key).expect("deserialize refresh token");
                    }
                    _ => panic!("")
                }
            }
        });
    }
}
