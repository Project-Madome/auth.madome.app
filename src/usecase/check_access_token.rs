use std::{collections::HashMap, convert::TryFrom, sync::Arc};

use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use util::{http::Cookie, ori};

use crate::{
    command::CommandSet,
    constant::http::cookie::MADOME_ACCESS_TOKEN,
    entity::{secret_key::SecretKey, token::AccessToken},
    error::UseCaseError,
    repository::{r#trait::SecretKeyRepository, RepositorySet},
};

pub struct Payload {
    pub access_token: String,
    pub minimum_role: Option<u8>,
    pub validate_exp: bool,
}

impl TryFrom<Request<Body>> for Payload {
    type Error = crate::Error;

    fn try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let mut cookie = Cookie::from(&request);

        let access_token = cookie.take(MADOME_ACCESS_TOKEN).unwrap_or_default();

        let qs = querystring::querify(request.uri().query().unwrap_or(""))
            .into_iter()
            .collect::<HashMap<_, _>>();

        let minimum_role = qs.get("role").and_then(|v| v.parse().ok());

        Ok(Self {
            access_token,
            minimum_role,
            validate_exp: true,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Model {
    #[serde(skip_serializing)]
    pub token_id: String,
    pub user_id: String,
}

impl From<Model> for Response<Body> {
    fn from(model: Model) -> Self {
        let serialized = serde_json::to_string(&model).expect("json serialize");

        Response::builder()
            .status(StatusCode::OK)
            .body(serialized.into())
            .unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unauthorized")]
    UnauthorizedAccessToken,
    #[error("Permission denied")]
    PermissionDenied,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

async fn deserialize(
    access_token: &str,
    validate_exp: bool,
    secret_key_repository: Arc<impl SecretKeyRepository>,
) -> crate::Result<Option<AccessToken>> {
    let token_id = ori!(AccessToken::deserialize_payload(access_token)).id;

    let SecretKey(secret_key) = ori!(secret_key_repository.get(&token_id).await?);

    let token_data = ori!(AccessToken::deserialize(
        access_token,
        &secret_key,
        validate_exp
    ))
    .claims;

    Ok(Some(token_data))
}

pub async fn execute(
    Payload {
        access_token,
        minimum_role,
        validate_exp,
    }: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let token_data = match deserialize(&access_token, validate_exp, repository.secret_key()).await?
    {
        Some(r) => r,
        None => return Err(Error::UnauthorizedAccessToken.into()),
    };

    if let Some(minimum_role) = minimum_role {
        let user = command.get_user_info(token_data.user_id.clone()).await?;

        if user.role < minimum_role {
            return Err(Error::PermissionDenied.into());
        }
    }

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

    use crate::command::{self, CommandSet};
    use crate::entity::token::Token;
    use crate::json::user::UserInfo;
    use crate::repository::{r#trait::SecretKeyRepository, RepositorySet};
    use crate::usecase::check_access_token::{self, Payload};

    #[tokio::test]
    async fn success() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: String, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4().to_string();
            token = Token::new(user_id.clone());

            repository
                .secret_key()
                .add(&token.id, &secret_key)
                .await
                .unwrap();
        },
        {
            let (serialized, _) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                access_token: serialized,
                minimum_role: None,
                validate_exp: true,
            };
            let r = check_access_token::execute(payload, repository, command)
                .await
                .unwrap();

            assert_eq!(r.token_id, token.id);
            assert_eq!(r.user_id, user_id);
        });
    }

    #[tokio::test]
    async fn success_with_permission_check() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: String, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4().to_string();
            token = Token::new(user_id.clone());

            repository
                .secret_key()
                .add(&token.id, &secret_key)
                .await
                .unwrap();

            let get_user_info = command::tests::GetUserInfo::from(UserInfo {
                id: user_id.clone(),
                email: "".to_string(),
                role: 1,
            });

            command.set_get_user_info(get_user_info);
        },
        {
            let (serialized, _) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                access_token: serialized,
                minimum_role: Some(0),
                validate_exp: true,
            };
            let r = check_access_token::execute(payload, repository, command)
                .await
                .unwrap();

            assert_eq!(r.token_id, token.id);
            assert_eq!(r.user_id, user_id);
        });
    }

    #[tokio::test]
    async fn error_permission_denied() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: String, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4().to_string();
            token = Token::new(user_id.clone());

            repository
                .secret_key()
                .add(&token.id, &secret_key)
                .await
                .unwrap();

            let get_user_info = command::tests::GetUserInfo::from(UserInfo {
                id: user_id.clone(),
                email: "".to_string(),
                role: 0,
            });

            command.set_get_user_info(get_user_info);
        },
        {
            let (serialized, _) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                access_token: serialized,
                minimum_role: Some(1),
                validate_exp: true,
            };
            let r = check_access_token::execute(payload, repository, command)
                .await
                .expect_err("expected error, but returns ok");


            assert_debug!(r, crate::Error::from(check_access_token::Error::PermissionDenied));
        });
    }

    #[tokio::test]
    async fn error_unauthorized_by_use_refresh_token_instead_of_access_token() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        // not same secret key
        // check_access_token::deserialize 함수에서 None을 리턴하면 Unauthorized 에러가 난다는 것만 테스트하면 됨
        test_registry!(
        [repository: RepositorySet, command: CommandSet] ->
        [secret_key: String, user_id: String, token: Token] ->
        {
            secret_key = "secret1234".to_string();
            user_id = Uuid::new_v4().to_string();
            token = Token::new(user_id.clone());

            repository
                .secret_key()
                .add(&token.id, &secret_key)
                .await
                .unwrap();

            let get_user_info = command::tests::GetUserInfo::from(UserInfo {
                id: user_id.clone(),
                email: "".to_string(),
                role: 0,
            });

            command.set_get_user_info(get_user_info);
        },
        {
            let (_, refresh_token) = token.serialize(&secret_key).expect("token serialize");

            let payload = Payload {
                access_token: refresh_token,
                minimum_role: None,
                validate_exp: true,
            };
            let r = check_access_token::execute(payload, repository, command)
                .await
                .expect_err("expected error, but returns ok");


            assert_debug!(r, crate::Error::from(check_access_token::Error::UnauthorizedAccessToken));
        });
    }
}
