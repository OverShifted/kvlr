use rustls::pki_types::CertificateDer;
use std::{io, sync::Arc};
use tokio_rustls::rustls::ServerConfig;

pub use tokio_rustls::server::TlsStream;

// Based on https://github.com/tmccombs/tls-listener/blob/main/examples/tls_config/mod.rs
pub fn acceptor(
    key: &mut dyn io::BufRead,
    cert: &mut dyn io::BufRead,
) -> tokio_rustls::TlsAcceptor {
    let key = rustls_pemfile::private_key(key).unwrap().unwrap();

    let cert: Vec<CertificateDer<'static>> = rustls_pemfile::certs(cert)
        .collect::<std::io::Result<_>>()
        .unwrap();

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .unwrap();

    config.alpn_protocols = vec![b"kvlr/0.1".to_vec()];

    Arc::new(config).into()
}
