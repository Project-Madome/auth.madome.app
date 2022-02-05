use sai::{Component, ComponentLifecycle, Injected};

use crate::config::Config;

#[derive(Component)]
#[lifecycle]
pub struct DatabaseSet {
    #[injected]
    config: Injected<Config>,

    redis: Option<redis::Client>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for DatabaseSet {
    async fn start(&mut self) {
        let redis = redis::Client::open(self.config.redis_url()).expect("connect redis");

        self.redis.replace(redis);
    }
}

impl DatabaseSet {
    pub async fn redis(&self) -> redis::RedisResult<redis::aio::Connection> {
        self.redis.as_ref().unwrap().get_async_connection().await
    }
}
