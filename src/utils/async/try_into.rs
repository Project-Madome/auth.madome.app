use super::AsyncTryFrom;

#[async_trait::async_trait]
pub trait AsyncTryInto<T>: Sized {
    type Error;

    async fn async_try_into(self) -> Result<T, Self::Error>;
}

#[async_trait::async_trait]
impl<T, U> AsyncTryInto<U> for T
where
    T: Send,
    U: AsyncTryFrom<T>,
{
    type Error = U::Error;

    async fn async_try_into(self) -> Result<U, Self::Error> {
        U::async_try_from(self).await
    }
}
