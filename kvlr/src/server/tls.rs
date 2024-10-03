use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;

pub use tokio_rustls::server::TlsStream;

pub fn acceptor(key: impl std::io::Read, cert: impl std::io::Read) -> tokio_rustls::TlsAcceptor {
    let key = PrivateKeyDer::from_pem_reader(key).unwrap();

    let cert: Vec<CertificateDer<'static>> = CertificateDer::pem_reader_iter(cert)
        .collect::<Result<_, _>>()
        .unwrap();

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .unwrap();

    config.alpn_protocols = vec![b"kvlr/0.1".to_vec()];

    Arc::new(config).into()
}
