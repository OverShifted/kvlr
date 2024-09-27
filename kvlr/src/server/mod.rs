pub mod tls;
// pub mod rpc_macro;

use crate::connection::Connection;
use crate::rpc::connection_state::HandlerFn;
use crate::utils::Unfold;
use anyhow::Context;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tls_listener::TlsListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info};

pub struct Server {
    listener: TlsListener<TcpListener, TlsAcceptor>,

    functions: Arc<RwLock<HashMap<u32, Arc<dyn HandlerFn>>>>,
}

impl Server {
    pub async fn new(
        key_path: &str,
        cert_path: &str,
        functions: Arc<RwLock<HashMap<u32, Arc<dyn HandlerFn>>>>,
    ) -> std::io::Result<Server> {
        Ok(Server {
            listener: tls_listener::builder(tls::acceptor(key_path, cert_path))
                .handshake_timeout(Duration::from_secs(10))
                .listen(TcpListener::bind("0.0.0.0:5857").await?),
            functions,
        })
    }

    pub async fn listen(&mut self) {
        info!("Listening.");
        let mut tasks = vec![];

        loop {
            tokio::select! {
                stream = self.listener.accept() => {
                    match stream.unwrap() {
                        Ok(stream) => {
                            stream.get_ref().0.set_nodelay(true).unwrap();
                            let functions = self.functions.clone();
                            tasks.push(tokio::spawn(async move {
                                Self::handle_connection(stream, functions).await
                            }))
                        }
                        Err(error) => error!("Could not accept connection: {}", error),
                    };
                },
                _ = tokio::signal::ctrl_c() => {
                    info!("Terminating.");
                    break;
                }
            }
        }
    }

    // TODO: Await for connection close
    async fn handle_connection(
        socket: impl AsyncReadExt + AsyncWriteExt + Send + Sync + Unpin + 'static,
        functions: Arc<RwLock<HashMap<u32, Arc<dyn HandlerFn>>>>,
    ) {
        let connection = Connection::new(Box::new(socket), functions);

        match tokio::time::timeout(Duration::from_secs(5), connection.recv_handshake())
            .await
            .context("Handshake failure: Handshake timed out.")
            .unfold()
        {
            Err(err) => error!("{:?}", err),
            Ok(()) => {
                // info!("Sucessful handshake.");
                let _ = connection.establish(3, 10).await;
            }
        }
    }
}

unsafe impl Sync for Server {}
