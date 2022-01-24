use nanoid::nanoid;
use sai::Component;

use crate::error::CommandError;

use super::r#trait::Command;

#[derive(Component)]
pub struct RandomCode;

impl r#trait::RandomCode for RandomCode {}

#[async_trait::async_trait]
impl Command<(), String> for RandomCode {
    type Error = crate::Error;

    async fn execute(&self, _: ()) -> Result<String, Self::Error> {
        Ok(nanoid!(12))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        CommandError::from(err).into()
    }
}

pub mod r#trait {
    use crate::command::r#trait::Command;

    pub trait RandomCode: Command<(), String, Error = crate::Error> {}
}
