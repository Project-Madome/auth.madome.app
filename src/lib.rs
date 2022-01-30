extern crate util;

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

pub use registry::RootRegistry;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

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
