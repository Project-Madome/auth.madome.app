#[async_trait::async_trait]
pub trait AsyncTryFrom<T>: Sized {
    type Error;

    async fn async_try_from(_: T) -> Result<Self, Self::Error>;
}
