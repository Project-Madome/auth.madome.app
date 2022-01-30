use crate::entity::authcode::Authcode;

#[async_trait::async_trait]
pub trait AuthcodeRepository: Send + Sync {
    async fn pop(&self, code: &str) -> crate::Result<Option<Authcode>>;

    async fn add(&self, authcode: Authcode) -> crate::Result<bool>;
}
