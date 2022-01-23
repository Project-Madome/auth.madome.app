use sai::System;
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    use madome_auth::RootRegistry;

    let mut system = System::<RootRegistry>::new();

    system.start().await;

    signal::ctrl_c().await.unwrap();

    // system.stop().await;
}
