use hyper::{Body, Response, StatusCode};
use util::http::{SetCookie, SetHeaders};

use crate::{
    command::{get_user_info, random_code, send_email},
    usecase::{
        check_access_token, check_and_refresh_token_pair, check_authcode, check_refresh_token,
        check_token_pair, create_authcode, create_token_pair, delete_token_pair,
        refresh_token_pair,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Msg: {0}")]
    Msg(#[from] Msg),
    #[error("Command: {0}")]
    Command(#[from] CommandError),
    #[error("UseCase: {0}")]
    UseCase(#[from] UseCaseError),
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),

    // TODO: 나중에 위치 재선정
    #[error("ReadChunksFromBody")]
    ReadChunksFromBody(#[from] hyper::Error),

    #[error("UserSdk: {0}")]
    UserSdk(#[from] madome_sdk::api::user::Error),

    #[cfg(test)]
    #[error("{0}")]
    Test(&'static str),
}

type Msg = crate::msg::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Redis: {0}")]
    Redis(#[from] redis::RedisError),
}

impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        Error::Repository(err.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("GetUserInfo: {0}")]
    GetUserInfo(#[from] get_user_info::Error),
    #[error("RandomCode: {0}")]
    RandomCode(#[from] random_code::Error),
    #[error("SendEmail: {0}")]
    SendEmail(#[from] send_email::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("CheckAccessToken: {0}")]
    CheckAccessToken(#[from] check_access_token::Error),
    #[error("CheckRefreshToken: {0}")]
    CheckRefreshToken(#[from] check_refresh_token::Error),
    #[error("CheckAuthcode: {0}")]
    CheckAuthcode(#[from] check_authcode::Error),
    #[error("CreateTokenPair: {0}")]
    CreateTokenPair(#[from] create_token_pair::Error),
    #[error("CheckTokenPair: {0}")]
    CheckTokenPair(#[from] check_token_pair::Error),
    #[error("CreateAuthcode: {0}")]
    CreateAuthcode(#[from] create_authcode::Error),
    #[error("CheckAndRefreshTokenPair: {0}")]
    CheckAndRefreshTokenPair(#[from] check_and_refresh_token_pair::Error),
    #[error("RefreshTokenPair: {0}")]
    RefreshTokenPair(#[from] refresh_token_pair::Error),
    #[error("DeleteTokenPair: {0}")]
    DeleteTokenPair(#[from] delete_token_pair::Error),
}

impl From<Error> for Response<Body> {
    fn from(error: Error) -> Self {
        use crate::msg::Error::*;
        use check_access_token::Error::*;
        use check_authcode::Error::*;
        use check_refresh_token::Error::*;
        use check_token_pair::Error::*;
        use create_authcode::Error::*;
        use Error::*;
        use UseCaseError::*;

        let response = Response::builder();

        // TODO: 복잡해지면 분리하자
        match error {
            Msg(JsonDeserializePayload(err)) => response
                .status(StatusCode::BAD_REQUEST)
                .body(err.to_string().into()),

            Msg(err @ NotFound) => response
                .status(StatusCode::NOT_FOUND)
                .body(err.to_string().into()),

            UseCase(CheckAccessToken(err @ PermissionDenied)) => response
                .status(StatusCode::FORBIDDEN)
                .body(err.to_string().into()),

            UseCase(CheckAccessToken(err @ UnauthorizedAccessToken)) => response
                .status(StatusCode::UNAUTHORIZED)
                .body(err.to_string().into()),

            UseCase(CheckRefreshToken(err @ UnauthorizedRefreshToken)) => response
                .status(StatusCode::UNAUTHORIZED)
                .body(err.to_string().into()),

            UseCase(CheckTokenPair(err @ InvalidTokenPair)) => response
                .status(StatusCode::UNAUTHORIZED)
                .body(err.to_string().into()),

            UseCase(CheckAuthcode(err @ InvalidAuthcode)) => response
                .status(StatusCode::NOT_FOUND)
                .body(err.to_string().into()),

            UseCase(CreateAuthcode(err @ TooManyCreatedAuthcode)) => response
                .status(StatusCode::TOO_MANY_REQUESTS)
                .body(err.to_string().into()),

            UseCase(CreateAuthcode(err @ create_authcode::Error::NotFoundUser)) => response
                .status(StatusCode::NOT_FOUND)
                .body(err.to_string().into()),

            UseCase(CreateTokenPair(err @ create_token_pair::Error::NotFoundUser)) => response
                .status(StatusCode::NOT_FOUND)
                .body(err.to_string().into()),

            UseCase(CheckAndRefreshTokenPair(
                err @ check_and_refresh_token_pair::Error::PermissionDenied(_),
            )) => {
                let err_str = err.to_string();
                let check_and_refresh_token_pair::Error::PermissionDenied(token_pair) = err;

                /* let t = match err {
                    check_and_refresh_token_pair::Error::PermissionDenied(t) => t,
                    // _ => unreachable!(),
                }; */

                response
                    .status(StatusCode::FORBIDDEN)
                    .headers(SetCookie::from(token_pair).iter())
                    .body(err_str.into())
            }

            UseCase(DeleteTokenPair(err @ delete_token_pair::Error::InvalidToken)) => response
                .status(StatusCode::BAD_REQUEST)
                .body(err.to_string().into()),

            UserSdk(ref err) => {
                use madome_sdk::api::{
                    user::{get_user, Error as UserError},
                    BaseError,
                };

                match err {
                    UserError::Base(err) => match err {
                        BaseError::Undefined(code, body) => {
                            response.status(code).body(body.to_string().into())
                        }
                        _ => response
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(err.to_string().into()),
                    },
                    UserError::GetUser(err) => match err {
                        get_user::Error::NotFoundUser => response
                            .status(StatusCode::NOT_FOUND)
                            .body(err.to_string().into()),
                    },
                    _ => response
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(err.to_string().into()),
                }
            }

            err => response
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(err.to_string().into()),
        }
        .unwrap()
    }
}
