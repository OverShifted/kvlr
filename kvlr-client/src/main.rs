mod client;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use kvlr::{
    client::request::Request,
    connection::Connection,
    promise_utils::PromiseHelper,
    streaming::{server::StreamRpc, stream_receiver::StreamReceiver},
};
use rustls::{
    pki_types::{CertificateDer, ServerName, UnixTime},
    SignatureScheme,
};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{
        client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
        ClientConfig, DigitallySignedStruct, Error,
    },
    TlsConnector,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug)]
struct TrueVerifier;

impl ServerCertVerifier for TrueVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            // SignatureScheme::RSA_PKCS1_SHA1,
            // SignatureScheme::ECDSA_SHA1_Legacy,
            // SignatureScheme::RSA_PKCS1_SHA256,
            // SignatureScheme::ECDSA_NISTP256_SHA256,
            // SignatureScheme::RSA_PKCS1_SHA384,
            // SignatureScheme::ECDSA_NISTP384_SHA384,
            // SignatureScheme::RSA_PKCS1_SHA512,
            // SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            // SignatureScheme::RSA_PSS_SHA384,
            // SignatureScheme::RSA_PSS_SHA512,
            // SignatureScheme::ED25519,
            // SignatureScheme::ED448
        ]
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let stream = TcpStream::connect("127.0.0.1:5857").await.unwrap();
    // stream.set_nodelay(true).unwrap();
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TrueVerifier))
        .with_no_client_auth();

    let config = TlsConnector::from(Arc::new(config));
    let domain = ServerName::try_from("127.0.0.1").unwrap();
    let stream = config.connect(domain, stream).await.unwrap();
    stream.get_ref().0.set_nodelay(true).unwrap();

    let functions = {
        let mut functions = HashMap::new();
        StreamRpc::register(&mut functions);
        Arc::new(RwLock::new(functions))
    };

    let connection = Connection::new(Box::new(stream), functions);
    connection.send_handshake().await.unwrap();

    let connection = connection.establish(3, 10).await;
    let rpc_manager = connection.create_rpc_manager().await;

    // TODO: Avoid hangs. maybe add a new CallID type?
    let call_id = client::Add { arg0: 10, arg1: 20 }
        .call_dropped(rpc_manager.clone())
        .await
        .unwrap();

    {
        let now = SystemTime::now();

        let res = client::AddPipelined {
            arg0: call_id.pipeline(),
            arg1: 20.into(),
        }
        .call(rpc_manager.clone())
        .await
        .unwrap();

        info!(?res, time=?now.elapsed().unwrap(), "AddPipelined");
    }

    let res = client::AppendString {
        arg0: "Hello ".to_string(),
        arg1: "World".to_string(),
    }
    .call(rpc_manager.clone())
    .await
    .unwrap();

    info!(res, "AppendString");

    let mut recv = StreamReceiver::<String>::new(42.into(), &connection, 10);
    tokio::spawn(async move {
        while let Ok(items_bytes) = recv.rx.recv().await {
            let items: Vec<String> = rmp_serde::from_slice(&items_bytes).unwrap();
            info!(?items, "Item(s) recved!");
        }
    });

    let res = client::RangeVec { arg0: 200 }
        .call(rpc_manager.clone())
        .await
        .unwrap();

    info!(?res, "RangeVec");

    client::CallMeToPanic.call(rpc_manager).on(
        |s| async move {
            info!(?s, "Yay!");
        },
        |e| async move {
            error!(?e, "Opps! Server returned an error!");
        },
    );

    tokio::time::sleep(Duration::from_millis(1000 * 5)).await;

    connection.close().await.unwrap();
    info!("Sent!");
}
