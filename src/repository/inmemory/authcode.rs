use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use sai::Component;
use util::ori;

use crate::{
    entity::authcode::{self, Authcode},
    repository::r#trait::AuthcodeRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct InMemoryAuthcodeRepository {
    inner: Mutex<HashMap<String, Vec<(Authcode, SystemTime)>>>,
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
                .find(|(_, (x, _))| x.code == code)
                .map(|(i, _)| i)
        };

        let (authcode, timer) = authcodes.remove(position);

        let expired =
            matches!(timer.elapsed(), Ok(elapsed) if elapsed.as_secs() > authcode::MAX_AGE);

        if expired {
            return Ok(None);
        }

        Ok(Some(authcode))
    }

    async fn add(&self, authcode: Authcode) -> crate::Result<bool> {
        let mut inner = self.inner.lock().unwrap();

        let authcodes = inner
            .entry(authcode.user_email.clone())
            .or_insert_with(|| Vec::with_capacity(5));

        if authcodes.len() >= 5 {
            let timer = authcodes.get(0).map(|(_, timer)| timer).unwrap();

            let expired =
                matches!(timer.elapsed(), Ok(elapsed) if elapsed.as_secs() > authcode::MAX_AGE);

            if !expired {
                return Ok(false);
            }

            authcodes.remove(0);
        }

        authcodes.push((authcode, SystemTime::now()));

        Ok(true)
    }
}
