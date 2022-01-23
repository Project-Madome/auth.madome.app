mod app;
mod command;
mod config;
mod constant;
mod database;
mod entity;
mod error;
mod json;
mod model;
mod msg;
mod registry;
mod repository;
mod usecase;
mod utils;

pub use registry::RootRegistry;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! tri {
    ($expr:expr) => {
        match $expr {
            Ok(msg) => msg,
            Err(error) => return error.into(),
        }
    };
}

/* #[async_trait::async_trait]
impl<A, B> AsyncTryInto<A> for B
where
    A: AsyncTryFrom<B>,
{
    type Error = <A as AsyncTryFrom<B>>::Error;

    async fn try_into(b: B) -> std::result::Result<A, Error> {
        <A as AsyncTryFrom<B>>::try_from(b).await
    }
} */
