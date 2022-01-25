use std::{collections::HashMap, sync::Mutex};

use sai::Component;

use crate::entity::authcode::Authcode;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[derive(Component)]
pub struct InMemoryAuthcodeRepository {
    inner: Mutex<HashMap<String, Authcode>>,
}

#[async_trait::async_trait]
impl r#trait::AuthcodeRepository for InMemoryAuthcodeRepository {
    async fn pop(&self, code: &str) -> crate::Result<Option<Authcode>> {
        let mut inner = self.inner.lock().unwrap();

        let authcode = inner.remove(code);

        Ok(authcode)
    }

    async fn add(&self, authcode: Authcode) -> crate::Result<bool> {
        let mut inner = self.inner.lock().unwrap();

        inner.insert(authcode.code.clone(), authcode);

        Ok(true)
    }
}

pub mod r#trait {
    use crate::entity::authcode::Authcode;

    #[async_trait::async_trait]
    pub trait AuthcodeRepository: Send + Sync {
        async fn pop(&self, code: &str) -> crate::Result<Option<Authcode>>;

        async fn add(&self, authcode: Authcode) -> crate::Result<bool>;
    }
}
