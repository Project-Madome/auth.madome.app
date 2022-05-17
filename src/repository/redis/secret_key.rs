use sai::{Component, Injected};
use uuid::Uuid;

use crate::{
    database::DatabaseSet, entity::secret_key::SecretKey, repository::r#trait::SecretKeyRepository,
};

#[derive(Component)]
pub struct RedisSecretKeyRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl SecretKeyRepository for RedisSecretKeyRepository {
    async fn get(&self, token_id: Uuid) -> crate::Result<Option<SecretKey>> {
        let mut redis = self.database.redis().await?;

        let secret_key: Option<String> = redis::cmd("GET")
            .arg(&[token_id.to_string()])
            .query_async(&mut redis)
            .await?;

        Ok(secret_key.map(SecretKey))
    }

    async fn add(&self, token_id: Uuid, secret_key: &str) -> crate::Result<bool> {
        let token_id = token_id.to_string();
        let mut redis = self.database.redis().await?;

        let r: bool = redis::cmd("SET")
            .arg(&[&token_id, secret_key])
            .query_async(&mut redis)
            .await?;

        log::debug!("added = {}", r);

        Ok(r)
    }

    async fn remove(&self, token_id: Uuid) -> crate::Result<bool> {
        let mut redis = self.database.redis().await?;

        let r: bool = redis::cmd("DEL")
            .arg(&[token_id.to_string()])
            .query_async(&mut redis)
            .await?;

        log::debug!("removed = {}", r);

        Ok(r)
    }
}
