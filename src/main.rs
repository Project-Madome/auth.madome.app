use madome_auth::RootRegistry;
use sai::System;
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    simple_logger::init().unwrap();

    let mut system = System::<RootRegistry>::new();

    system.start().await;

    signal::ctrl_c().await.unwrap();

    // system.stop().await;
}
