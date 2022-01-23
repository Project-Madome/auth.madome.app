use sai::{Component, Injected};

use crate::{config::Config, error::CommandError};

use super::r#trait::Command;

#[derive(Component)]
pub struct SendEmail {
    #[injected]
    config: Injected<Config>,
}

#[async_trait::async_trait]
impl Command<(String, String), ()> for SendEmail {
    type Error = crate::Error;

    async fn execute(&self, (email, content): (String, String)) -> Result<(), Self::Error> {
        unimplemented!()
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

    pub trait SendEmail: Command<(String, String), (), Error = crate::Error> {}
}

#[cfg(test)]
pub mod tests {
    use sai::Component;

    use crate::command::r#trait::Command;

    use super::r#trait;

    #[derive(Component)]
    pub struct SendEmail;

    impl r#trait::SendEmail for SendEmail {}

    #[async_trait::async_trait]
    impl Command<(String, String), ()> for SendEmail {
        type Error = crate::Error;

        async fn execute(&self, _: (String, String)) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}
