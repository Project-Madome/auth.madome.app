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

#[derive(Component)]
#[lifecycle]
pub struct Config {
    port: Option<u16>,

    madome_user_server: Option<String>,

    secret_key: Option<String>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for Config {
    async fn start(&mut self) {
        dotenv::dotenv().ok();

        self.port.replace(env("PORT"));

        self.madome_user_server.replace(env("MADOME_USER_SERVER"));

        self.secret_key.replace(env("SECRET_KEY"));
    }
}

impl Config {
    pub fn port(&self) -> u16 {
        self.port.unwrap()
    }

    pub fn madome_user_server(&self) -> &str {
        self.madome_user_server.as_ref().unwrap()
    }

    pub fn secret_key(&self) -> &str {
        self.secret_key.as_ref().unwrap()
    }
}
