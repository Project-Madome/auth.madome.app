use std::sync::Arc;

use either::Either;
use hyper::{Body, Request};
use serde::Deserialize;
use util::{r#async::AsyncTryFrom, validate::ValidatorStringExt, FromOwnedRequest};

use crate::{
    command::CommandSet,
    entity::authcode::Authcode,
    error::UseCaseError,
    msg::Wrap,
    repository::{r#trait::AuthcodeRepository, RepositorySet},
};

#[derive(Deserialize)]
pub struct Payload {
    #[serde(rename = "email")]
    pub user_email: String,

    /// use this only debug build and e2e testing
    /// true => send email
    /// false => don't send email
    #[serde(default)]
    pub ses_flag: bool,
}

impl Payload {
    fn check(self) -> crate::Result<Self> {
        let user_email = self
            .user_email
            .validate()
            .email()
            .take()
            .map_err(|_| Error::InvalidEmail)?;

        Ok(Self { user_email, ..self })
    }
}

#[async_trait::async_trait]
impl FromOwnedRequest for Payload {
    type Error = crate::Error;
    type Parameter = ();

    async fn from_owned_request(
        _parameter: Self::Parameter,
        request: Request<Body>,
    ) -> Result<Self, Self::Error> {
        #[cfg(debug_assertions)]
        let ses_flag = request
            .headers()
            .get(madome_sdk::api::header::MADOME_E2E_TEST)
            .is_none();

        #[cfg(debug_assertions)]
        log::debug!("ses_flag = {ses_flag}");

        let payload: Self = Wrap::async_try_from(request).await?.inner();

        Ok(Self {
            #[cfg(debug_assertions)]
            ses_flag,
            ..payload.check()?
        })
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid email")]
    InvalidEmail,

    #[error("Not found user")]
    NotFoundUser,

    #[error("Too many created authcode")]
    TooManyCreatedAuthcode,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload {
        user_email,
        ses_flag,
    }: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let user = command.get_user_info(Either::Right(user_email)).await?;

    let authcode_repository = repository.authcode();

    let code = command.random_code().await?;

    let authcode = Authcode::new(user.email.clone(), code.clone());

    log::debug!("created authcode = {}", authcode.code);

    let created = authcode_repository.add(authcode).await?;

    if !created {
        return Err(Error::TooManyCreatedAuthcode.into());
    }

    #[cfg(feature = "aws-ses")]
    {
        #[cfg(not(debug_assertions))]
        {
            command.send_email(user.email, code).await?;
        }
        #[cfg(debug_assertions)]
        {
            if ses_flag {
                command.send_email(user.email.clone(), code.clone()).await?;
            }
        }
    }

    // e2e channel server??? ??????
    #[cfg(debug_assertions)]
    {
        // tokio::spawn(async move {
        #[derive(serde::Serialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum Command {
            Authcode { email: String, code: String },
        }

        if let Ok(debug_url) = std::env::var("E2E_CHANNEL_URL") {
            let serialized = serde_json::to_vec(&Command::Authcode {
                email: user.email,
                code,
            });

            if let Ok(serialized) = serialized {
                let _resp = reqwest::Client::new()
                    .put(debug_url)
                    .body(serialized)
                    .send()
                    .await
                    .ok();
            }
        }
        // });
    }

    Ok(Model)
}

#[cfg(test)]
mod tests {
    // TODO: success(send_email??? authcode??? ???????????? ?????? ?????? random_code??? ?????? ?????? ????????????)
}
