use rustls::pki_types::CertificateDer;
use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;

pub use tokio_rustls::server::TlsStream;

// Based on https://github.com/tmccombs/tls-listener/blob/main/examples/tls_config/mod.rs
pub fn acceptor(key_path: &str, cert_path: &str) -> tokio_rustls::TlsAcceptor {
    let key = {
        let file = std::fs::File::open(key_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        rustls_pemfile::private_key(&mut reader).unwrap().unwrap()
    };

    let cert: Vec<CertificateDer<'static>> = {
        let file = std::fs::File::open(cert_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        rustls_pemfile::certs(&mut reader)
            .collect::<std::io::Result<Vec<CertificateDer<'static>>>>()
            .unwrap()
    };

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .unwrap();

    // config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config.alpn_protocols = vec![b"http/1.1".to_vec()];

    Arc::new(config).into()
}
