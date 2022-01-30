use std::{collections::HashMap, sync::RwLock};

use chrono::Utc;
use sai::Component;

use crate::{
    entity::secret_key::{SecretKey, SECRET_KEY_EXP},
    repository::r#trait::SecretKeyRepository,
};

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct InMemorySecretKeyRepository {
    inner: RwLock<HashMap<String, (SecretKey, i64)>>,
}

#[async_trait::async_trait]
impl SecretKeyRepository for InMemorySecretKeyRepository {
    async fn get(&self, token_id: &str) -> crate::Result<Option<SecretKey>> {
        let inner = self.inner.read().unwrap();

        let now = Utc::now().timestamp();

        match inner.get(token_id).cloned() {
            Some((secret_key, created_at)) if now - created_at <= SECRET_KEY_EXP => {
                Ok(Some(secret_key))
            }
            _ => Ok(None),
        }
    }

    async fn add(&self, token_id: &str, secret_key: &str) -> crate::Result<bool> {
        let mut inner = self.inner.write().unwrap();

        let created_at = Utc::now().timestamp();

        inner.insert(
            token_id.to_string(),
            (SecretKey(secret_key.to_string()), created_at),
        );

        Ok(true)
    }

    async fn remove(&self, token_id: &str) -> crate::Result<bool> {
        let mut inner = self.inner.write().unwrap();

        inner.remove(token_id);

        Ok(true)
    }
}
