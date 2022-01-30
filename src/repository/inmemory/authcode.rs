use std::{collections::HashMap, sync::Mutex};

use sai::Component;

use crate::{entity::authcode::Authcode, repository::r#trait::AuthcodeRepository};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct InMemoryAuthcodeRepository {
    inner: Mutex<HashMap<String, Authcode>>,
}

#[async_trait::async_trait]
impl AuthcodeRepository for InMemoryAuthcodeRepository {
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
