use hyper::{Body, Response, StatusCode};

use crate::{
    command::{get_user_info, random_code, send_email},
    repository::authcode_repository,
    usecase::{
        check_access_token, check_authcode, check_refresh_token, check_token_pair, create_authcode,
        create_token_pair,
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
}

type Msg = crate::msg::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Authcode: {0}")]
    Authcode(#[from] authcode_repository::Error),
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
}

impl From<Error> for Response<Body> {
    fn from(error: Error) -> Self {
        use crate::msg::Error::*;
        use check_access_token::Error::*;
        use check_authcode::Error::*;
        use check_refresh_token::Error::*;
        use check_token_pair::Error::*;
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

            err => response
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(err.to_string().into()),
        }
        .unwrap()
    }
}
