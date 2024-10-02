mod server_impl;
#[rustfmt::skip]
mod server_trait;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use kvlr::{server::Server, utils::array_buf_read};
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

    let mut key_bytes = array_buf_read(include_bytes!("../../keys/server.key"));
    let mut cert_bytes = array_buf_read(include_bytes!("../../keys/server.crt"));

    Server::new(
        &mut key_bytes,
        &mut cert_bytes,
        Arc::new(RwLock::new(functions)),
    )
    .await?
    .listen()
    .await;

    Ok(())
}
