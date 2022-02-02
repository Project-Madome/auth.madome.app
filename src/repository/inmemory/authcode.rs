use std::{collections::HashMap, sync::Mutex};

use sai::Component;
use util::ori;

use crate::{entity::authcode::Authcode, repository::r#trait::AuthcodeRepository};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct InMemoryAuthcodeRepository {
    inner: Mutex<HashMap<String, Vec<Authcode>>>,
}

#[async_trait::async_trait]
impl AuthcodeRepository for InMemoryAuthcodeRepository {
    async fn pop(&self, user_email: &str, code: &str) -> crate::Result<Option<Authcode>> {
        let mut inner = self.inner.lock().unwrap();

        let authcodes = ori!(inner.get_mut(user_email));

        let position = ori! {
            authcodes
                .iter()
                .enumerate()
                .find(|(_, x)| x.code == code)
                .map(|(i, _)| i)
        };

        let authcode = authcodes.remove(position);

        Ok(Some(authcode))
    }

    async fn add(&self, authcode: Authcode) -> crate::Result<bool> {
        let mut inner = self.inner.lock().unwrap();

        let authcodes = inner
            .entry(authcode.user_email.clone())
            .or_insert_with(|| Vec::with_capacity(5));

        if authcodes.len() >= 5 {
            let expired = authcodes.get(0).map(|x| x.expired()).unwrap();

            if !expired {
                return Ok(false);
            }

            authcodes.remove(0);
        }

        authcodes.push(authcode);

        Ok(true)
    }
}
