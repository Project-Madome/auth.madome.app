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

/* from_another_error![
    (UseCase, CheckAccessToken, check_access_token::Error),
    (UseCase, CheckAuthcode, check_authcode::Error),
    (UseCase, CheckTokenPair, check_token_pair::Error),
];

#[macro_export]
macro_rules! from_another_error {
    ($(($first_member:ident, $second_member:ident, $from:ty)),*,) => {
        $(
            impl From<$from> for Error {
                fn from(error: $from) -> Self {
                    Self::$first_member($first_member::$second_member(error))
                }
            }
        )*
    };
} */

// TODO: 나중에 에러 핸들링 레이어 구현부 쪽에서 처리하자
/* impl From<Error> for Response<Body> {
    fn from(error: Error) -> Self {
        match error {
            Error::NotFound => not_found(),
            Error::JsonDeserialize(_e) => bad_request(),
            Error::ReadChunksFromBody(_e) => internal_server_error(),
        }
    }
} */
/*
fn not_found() -> Response<Body> {
    Response::builder().status(404).body(Body::empty()).unwrap()
}

fn bad_request() -> Response<Body> {
    Response::builder().status(400).body(Body::empty()).unwrap()
}

fn internal_server_error() -> Response<Body> {
    Response::builder().status(500).body(Body::empty()).unwrap()
} */
