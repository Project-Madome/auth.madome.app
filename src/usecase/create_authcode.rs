use std::sync::Arc;

use serde::Deserialize;

use crate::{
    command::CommandSet,
    entity::authcode::Authcode,
    error::UseCaseError,
    repository::{r#trait::AuthcodeRepository, RepositorySet},
};

#[derive(Deserialize)]
pub struct Payload {
    #[serde(rename = "email")]
    pub user_email: String,
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

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
    let code = command.random_code().await?;

    let authcode = Authcode::new(user_email.clone(), code.clone());

    log::debug!("created authcode = {}", authcode.code);

    let _ = repository.authcode().add(authcode).await?;

    command.send_email(user_email, code).await?;

    Ok(Model)
}

#[cfg(test)]
mod tests {
    // TODO: success(send_email이 authcode를 반환하게 하고 그걸 random_code에 넣은 값과 비교하자)
}
