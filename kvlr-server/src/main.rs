// #![feature(async_closure)]

mod server_trait;
mod server_impl;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use kvlr::server::Server;
use server_impl::ServerImpl;
use server_trait::SomeFunctions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let mut functions = HashMap::new();
    ServerImpl::register(Arc::new(ServerImpl), &mut functions);

    Server::new(
        "key.pem",
        "cert.pem",
        Arc::new(RwLock::new(functions))
    )
    .await?
    .listen()
    .await;

    Ok(())
}
