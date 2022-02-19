use std::sync::Arc;

use serde::Deserialize;

use crate::{
    command::CommandSet,
    entity::authcode::Authcode,
    error::UseCaseError,
    release,
    repository::{r#trait::AuthcodeRepository, RepositorySet},
};

#[derive(Deserialize)]
pub struct Payload {
    #[serde(rename = "email")]
    pub user_email: String,
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    Payload { user_email }: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    let user = match command.get_user_info(user_email).await? {
        Some(user) => user,
        None => return Err(Error::NotFoundUser.into()),
    };

    let authcode_repository = repository.authcode();

    let code = command.random_code().await?;

    let authcode = Authcode::new(user.email.clone(), code.clone());

    log::debug!("created authcode = {}", authcode.code);

    let created = authcode_repository.add(authcode).await?;

    if !created {
        return Err(Error::TooManyCreatedAuthcode.into());
    }

    #[cfg(not(debug_assertions))]
    {
        command.send_email(user.email, code).await?;
    }

    // e2e channel server에 보냄
    #[cfg(debug_assertions)]
    {
        // tokio::spawn(async move {
        #[derive(serde::Serialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum Command {
            Authcode { email: String, code: String },
        }

        if let Ok(debug_url) = std::env::var("DEBUG_URL") {
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
    // TODO: success(send_email이 authcode를 반환하게 하고 그걸 random_code에 넣은 값과 비교하자)
}
