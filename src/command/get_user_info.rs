use sai::{Component, Injected};

use crate::{command::r#trait::Command, config::Config, error::CommandError, json::user::UserInfo};

#[derive(Component)]
pub struct GetUserInfo {
    #[injected]
    config: Injected<Config>,
}

impl r#trait::GetUserInfo for GetUserInfo {}

#[async_trait::async_trait]
impl Command<String, UserInfo> for GetUserInfo {
    type Error = crate::Error;

    async fn execute(&self, user_id_or_email: String) -> Result<UserInfo, Self::Error> {
        let url = format!(
            "{}/users/{}",
            self.config.madome_user_server(),
            user_id_or_email
        );

        let res = reqwest::get(url).await.map_err(Error::from)?;

        let user_info = res.json::<UserInfo>().await.map_err(Error::from)?;

        Ok(user_info)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    Reqwest(#[from] reqwest::Error),
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        CommandError::from(err).into()
    }
}

pub mod r#trait {
    use crate::{command::r#trait::Command, json::user::UserInfo};

    pub trait GetUserInfo: Command<String, UserInfo, Error = crate::Error> {}
}

#[cfg(test)]
pub mod tests {
    // test command implements here..

    use sai::Component;

    use crate::{command::r#trait::Command, json::user::UserInfo};

    use super::r#trait;

    #[derive(Component, Default)]
    pub struct GetUserInfo {
        users: Vec<UserInfo>,
    }

    impl From<UserInfo> for GetUserInfo {
        fn from(user: UserInfo) -> Self {
            Self { users: vec![user] }
        }
    }

    impl r#trait::GetUserInfo for GetUserInfo {}

    #[async_trait::async_trait]
    impl Command<String, UserInfo> for GetUserInfo {
        type Error = crate::Error;

        async fn execute(&self, id_or_email: String) -> Result<UserInfo, Self::Error> {
            let user = self
                .users
                .iter()
                .find(|user| user.id == id_or_email || user.email == id_or_email)
                .unwrap();
            Ok(user.clone())
        }
    }
}
