mod server_impl;
#[rustfmt::skip]
mod server_trait;

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, RwLock},
};

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

    let key_bytes = include_bytes!("../../keys/server.key");
    let cert_bytes = include_bytes!("../../keys/server.crt");

    Server::new(
        Cursor::new(key_bytes),
        Cursor::new(cert_bytes),
        Arc::new(RwLock::new(functions)),
    )
    .await?
    .listen()
    .await;

    Ok(())
}
