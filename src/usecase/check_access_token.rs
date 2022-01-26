use std::{convert::TryFrom, sync::Arc};

use hyper::{Body, Request, Response, StatusCode};

use crate::{
    command::CommandSet, constant::http::cookie::MADOME_ACCESS_TOKEN, entity::token::AccessToken,
    error::UseCaseError, utils::http::Cookie,
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

        // TODO: QueryParameterParser 구현
        // let minimu_role = queryparameter.minimum_role;

        Ok(Self {
            access_token,
            minimum_role: None,
            validate_exp: true,
        })
    }
}

pub type Model = AccessToken;

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

pub async fn execute(
    Payload {
        access_token,
        minimum_role,
        validate_exp, // Refresh할때만 true
    }: Payload,
    command: Arc<CommandSet>,
    secret_key: &str,
) -> crate::Result<Model> {
    let token_data = match AccessToken::deserialize(&access_token, secret_key, validate_exp) {
        Some(r) => r.claims,
        None => return Err(Error::UnauthorizedAccessToken.into()),
    };

    if let Some(minimum_role) = minimum_role {
        let user = command.get_user_info(token_data.user_id.clone()).await?;

        if user.role < minimum_role {
            return Err(Error::PermissionDenied.into());
        }
    }

    Ok(token_data)
}

#[cfg(test)]
mod tests {
    //
}
