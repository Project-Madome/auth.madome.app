use crate::{entity::token::RefreshToken, error::UseCaseError};

pub struct Payload {
    pub refresh_token: String,
}

pub struct Model {
    pub token: RefreshToken,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload { refresh_token }: Payload,
    // command: Arc<CommandSet>,
    secret_key: &str,
) -> crate::Result<Model> {
    let token_data = RefreshToken::deserialize(&refresh_token, secret_key)?.claims;

    Ok(Model { token: token_data })
}

#[cfg(test)]
mod tests {
    //
}
