use either::Either;
use madome_sdk::api::user::{get_user, model};
use sai::{Component, Injected};
use uuid::Uuid;

use crate::{command::r#trait::Command, config::Config, error::CommandError};

#[derive(Component)]
pub struct GetUser {
    #[injected]
    config: Injected<Config>,
}

impl r#trait::GetUser for GetUser {}

#[async_trait::async_trait]
impl Command<Either<Uuid, String>, model::User> for GetUser {
    type Error = crate::Error;

    async fn execute(
        &self,
        user_id_or_email: Either<Uuid, String>,
    ) -> Result<model::User, Self::Error> {
        /* let url = format!(
            "{}/users/{}",
            self.config.madome_user_server(),
            user_id_or_email
        );

        let res = reqwest::get(url).await.map_err(Error::from)?;

        if res.status().as_u16() == 404 {
            return Ok(None);
        }

        let user_info = res.json::<UserInfo>().await.map_err(Error::from)?; */

        let user = get_user(self.config.madome_user_url(), "", user_id_or_email).await?;

        Ok(user)
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
    use either::Either;
    use madome_sdk::api::user::model;
    use uuid::Uuid;

    use crate::command::r#trait::Command;

    pub trait GetUser: Command<Either<Uuid, String>, model::User, Error = crate::Error> {}
}

#[cfg(test)]
pub mod tests {
    // test command implements here..

    use either::Either;
    use madome_sdk::api::user::model;
    use sai::Component;
    use uuid::Uuid;

    use crate::command::r#trait::Command;

    use super::r#trait;

    #[derive(Component, Default)]
    pub struct GetUser {
        users: Vec<model::User>,
    }

    impl From<model::User> for GetUser {
        fn from(user: model::User) -> Self {
            Self { users: vec![user] }
        }
    }

    impl r#trait::GetUser for GetUser {}

    #[async_trait::async_trait]
    impl Command<Either<Uuid, String>, model::User> for GetUser {
        type Error = crate::Error;

        async fn execute(
            &self,
            id_or_email: Either<Uuid, String>,
        ) -> Result<model::User, Self::Error> {
            let user = self.users.iter().find(|user| match &id_or_email {
                Either::Left(user_id) => user_id == &user.id,
                Either::Right(user_email) => user_email == &user.email,
            });

            match user {
                Some(user) => Ok(user.clone()),
                None => Err(crate::Error::Test("not found user")),
            }
        }
    }
}
