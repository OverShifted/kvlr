use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;

pub use tokio_rustls::server::TlsStream;

// Based on https://github.com/tmccombs/tls-listener/blob/main/examples/tls_config/mod.rs
pub fn acceptor(key_path: &str, cert_path: &str) -> tokio_rustls::TlsAcceptor {
    let key = {
        let file = std::fs::File::open(key_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        let keys = rustls_pemfile::rsa_private_keys(&mut reader).unwrap();
        tokio_rustls::rustls::PrivateKey(keys[0].clone())
    };

    let cert = {
        let file = std::fs::File::open(cert_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        rustls_pemfile::certs(&mut reader)
            .unwrap()
            .into_iter()
            .map(tokio_rustls::rustls::Certificate)
            .collect()
    };

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .unwrap();

    // config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config.alpn_protocols = vec![b"http/1.1".to_vec()];

    Arc::new(config).into()
}
