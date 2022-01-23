pub mod get_user_info;
pub mod random_code;
pub mod send_email;

pub use get_user_info::GetUserInfo;
pub use random_code::RandomCode;
pub use send_email::SendEmail;

use sai::{Component, Injected};

use crate::json::user::UserInfo;

use self::r#trait::Command;

pub mod r#trait {
    pub use super::get_user_info::r#trait::GetUserInfo;

    /// 인자가 여러개라면 Command<(String, u8, i8, u32), String> 이런식으로
    #[async_trait::async_trait]
    pub trait Command<T, R> {
        type Error;

        async fn execute(&self, _: T) -> Result<R, Self::Error>;
    }
}

#[derive(Component)]
pub struct CommandSet {
    #[cfg(not(test))]
    #[injected]
    get_user_info: Injected<GetUserInfo>,

    #[cfg(test)]
    #[injected]
    get_user_info: Injected<tests::GetUserInfo>,

    #[injected]
    random_code: Injected<RandomCode>,

    #[cfg(not(test))]
    #[injected]
    send_email: Injected<SendEmail>,

    #[cfg(test)]
    #[injected]
    send_email: Injected<tests::SendEmail>,
}

impl CommandSet {
    pub async fn get_user_info(&self, user_id_or_email: String) -> crate::Result<UserInfo> {
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
    pub use super::get_user_info::tests::*;
    pub use super::send_email::tests::*;
}

/* macro_rules! command_traits {
    ($name:tt, [$($generic:tt),+]) => {
        #[async_trait::async_trait]
        pub trait $name<$($generic,)* R> {
            type Error;

            async fn execute($(_: $generic,)*) -> Result<R, Self::Error>;
        }
    };
}

command_traits!(Command1, [T]);
command_traits!(Command2, [T1, T2]);
command_traits!(Command3, [T1, T2, T3]);
command_traits!(Command4, [T1, T2, T3, T4]);
command_traits!(Command5, [T1, T2, T3, T4, T5]);

/// command(T1 -> T2 -> R, fn)
macro_rules! command {
    ($name:tt, $arg1:ty => $r:ty, $expr:expr) => {
        pub struct $name;

        #[async_trait::async_trait]
        impl Command1<$arg1, $r> for $name {
            type Error = crate::Error;

            async fn execute(arg1: $arg1) -> Result<$r, Self::Error> {
                ($expr(arg1)).await
            }
        }
    };
}

command! {
    GetUserInfo,
    &str => crate::json::user::UserInfo,
    |email: &str| async {
        Ok(crate::json::user::UserInfo {
            id: "",
            email: "",
            role: 0,
        })
    }
}

async fn test() {
    GetUserInfo::execute("arg0").await
}
 */
