mod server_impl;
#[rustfmt::skip]
mod server_trait;
mod models;
mod schema;

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, RwLock},
};

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use kvlr::{server::Server, streaming::server::StreamRpc};
use server_impl::ServerImpl;
use server_trait::SomeFunctions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // console_subscriber::init();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let mut functions = HashMap::new();
    StreamRpc::register(&mut functions);
    ServerImpl::register(Arc::new(ServerImpl::new()), &mut functions);

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
