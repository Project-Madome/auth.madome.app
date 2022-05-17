use uuid::Uuid;

use crate::entity::secret_key::SecretKey;

#[async_trait::async_trait]
pub trait SecretKeyRepository: Send + Sync {
    async fn get(&self, token_id: Uuid) -> crate::Result<Option<SecretKey>>;

    async fn add(&self, token_id: Uuid, secret_key: &str) -> crate::Result<bool>;

    async fn remove(&self, token_id: Uuid) -> crate::Result<bool>;
}
