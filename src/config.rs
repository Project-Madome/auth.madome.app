use std::{env, fmt::Debug, str::FromStr};

use sai::{Component, ComponentLifecycle};

fn env<T>(key: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let var = env::var(key).expect("Please set dotenv");

    var.parse().expect("Please set dotenv to valid value")
}

#[derive(Debug, Component)]
#[lifecycle]
pub struct Config {
    port: Option<u16>,

    redis_url: Option<String>,

    madome_user_server: Option<String>,

    // AWS_ACCESS_KEY_ID=
    // AWS_SECRET_ACCESS_KEY=
    aws_config: Option<aws_config::Config>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for Config {
    async fn start(&mut self) {
        dotenv::dotenv().ok();

        self.port.replace(env("PORT"));

        self.redis_url.replace(env("REDIS_URL"));

        self.madome_user_server.replace(env("MADOME_USER_SERVER"));

        self.aws_config
            .replace(aws_config::from_env().region("us-east-1").load().await);

        log::info!("{:?}", self);
    }
}

impl Config {
    pub fn port(&self) -> u16 {
        self.port.unwrap()
    }

    pub fn redis_url(&self) -> &str {
        self.redis_url.as_ref().unwrap()
    }

    pub fn madome_user_server(&self) -> &str {
        self.madome_user_server.as_ref().unwrap()
    }

    pub fn aws_config(&self) -> &aws_config::Config {
        self.aws_config.as_ref().unwrap()
    }
}
