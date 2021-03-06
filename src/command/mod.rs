pub mod get_user_info;
pub mod random_code;
pub mod send_email;

use either::Either;
pub use get_user_info::GetUser;
pub use random_code::RandomCode;
pub use send_email::SendEmail;

use madome_sdk::api::user::model;
use sai::{Component, Injected};
use uuid::Uuid;

use self::r#trait::Command;

pub mod r#trait {
    pub use super::get_user_info::r#trait::GetUser;

    /// 인자가 여러개라면 Command<(String, u8, i8, u32), String> 이런식으로
    #[async_trait::async_trait]
    pub trait Command<T, R> {
        type Error;

        async fn execute(&self, _: T) -> Result<R, Self::Error>;
    }
}

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct CommandSet {
    #[cfg(not(test))]
    #[injected]
    get_user_info: Injected<GetUser>,

    #[cfg(test)]
    #[injected]
    get_user_info: Injected<tests::GetUser>,

    #[cfg(not(test))]
    #[injected]
    random_code: Injected<RandomCode>,

    #[cfg(test)]
    #[injected]
    random_code: Injected<tests::RandomCode>,

    #[cfg(not(test))]
    #[injected]
    send_email: Injected<SendEmail>,

    #[cfg(test)]
    #[injected]
    send_email: Injected<tests::SendEmail>,
}

impl CommandSet {
    pub async fn get_user_info(
        &self,
        user_id_or_email: Either<Uuid, String>,
    ) -> crate::Result<model::User> {
        self.get_user_info.execute(user_id_or_email).await
    }

    pub async fn random_code(&self) -> crate::Result<String> {
        self.random_code.execute(()).await
    }

    pub async fn send_email(&self, email: String, content: String) -> crate::Result<()> {
        self.send_email.execute((email, content)).await
    }
}

#[cfg(test)]
pub mod tests {

    use sai::Injected;

    pub use super::get_user_info::tests::*;
    pub use super::random_code::tests::*;
    pub use super::send_email::tests::*;

    impl super::CommandSet {
        pub fn set_get_user_info(&mut self, r: GetUser) {
            self.get_user_info = Injected::new(r);
        }
    }
}
