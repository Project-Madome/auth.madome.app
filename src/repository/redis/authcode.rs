use futures_util::StreamExt;
use redis::AsyncCommands;
use sai::{Component, Injected};

use crate::{
    database::DatabaseSet,
    entity::authcode::{self, Authcode},
    repository::r#trait::AuthcodeRepository,
};

#[derive(Component)]

pub struct RedisAuthcodeRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl AuthcodeRepository for RedisAuthcodeRepository {
    async fn pop(&self, user_email: &str, code: &str) -> crate::Result<Option<Authcode>> {
        let mut redis = self.database.redis().await?;

        let key = format!("authcode:{}:{}", user_email, code);

        let r: Option<String> = redis::cmd("GETDEL")
            .arg(&[&key])
            .query_async(&mut redis)
            .await?;

        Ok(r.map(|code| Authcode {
            code,
            user_email: user_email.to_string(),
        }))
    }

    async fn add(&self, Authcode { user_email, code }: Authcode) -> crate::Result<bool> {
        let mut redis = self.database.redis().await?;

        let pattern = format!("authcode:{}:*", user_email);
        let keys = redis
            .scan_match(&pattern)
            .await?
            .collect::<Vec<String>>()
            .await;

        log::debug!("scan_match({}) = {:?}", pattern, keys);

        if keys.len() >= 5 {
            return Ok(false);
        }

        let key = format!("authcode:{}:{}", user_email, code);
        let max_age = authcode::MAX_AGE;

        /* let r: bool = redis::cmd("SETEX")
        .arg(&[&key, &max_age, &code])
        .query_async(&mut redis)
        .await?; */

        let r = redis.set_ex(key, code, max_age as usize).await?;

        log::debug!("r = {r}");

        Ok(r)
    }
}
