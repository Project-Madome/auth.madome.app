/* use std::{convert::TryFrom, sync::Arc};

use hyper::{Body, Request};
use serde::Deserialize;

use crate::{
    command::CommandSet,
    constant::http::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN},
    utils::http::Cookie,
};

use super::{check_token_pair, create_token_pair};

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

pub struct Model {
    pub access_token: String,
    pub refresh_token: String,
}

impl From<create_token_pair::Model> for Model {
    fn from(m: create_token_pair::Model) -> Self {
        Self {
            access_token: m.access_token,
            refresh_token: m.refresh_token,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub async fn execute(
    old_token_pair: Payload,
    command: Arc<CommandSet>,
    secret_key: &str,
) -> Result<Model, UseCaseError> {
    let user_id =
        check_token_pair::execute(old_token_pair.into(), Arc::clone(&command), secret_key).await?;

    let refreshed_token_pair =
        create_token_pair::execute(user_id.into(), command, secret_key).await?;

    Ok(refreshed_token_pair.into())
}
 */
