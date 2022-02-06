use log::Level;
use madome_auth::{release, RootRegistry};
use sai::System;
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let log_level = if release() { Level::Info } else { Level::Debug };

    simple_logger::init_with_level(log_level).unwrap();

    let mut system = System::<RootRegistry>::new();

    system.start().await;

    signal::ctrl_c().await.unwrap();

    // system.stop().await;
}
