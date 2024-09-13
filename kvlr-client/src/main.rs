mod client;

use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use std::net::IpAddr;

use kvlr::{client::request::Request, connection::Connection};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{
        client::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
        Certificate, ClientConfig, DigitallySignedStruct, Error, ServerName,
    },
    TlsConnector,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

struct TrueVerifier;

impl ServerCertVerifier for TrueVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let stream = TcpStream::connect("127.0.0.1:5857").await.unwrap();
    // stream.set_nodelay(true).unwrap();
    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(TrueVerifier))
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(TrueVerifier));
    let config = TlsConnector::from(Arc::new(config));
    let domain = ServerName::IpAddress(IpAddr::from_str("127.0.0.1").unwrap());
    let stream = config.connect(domain, stream).await.unwrap();
    stream.get_ref().0.set_nodelay(true).unwrap();

    let mut connection = Connection::new(Box::new(stream), Default::default());
    connection.send_handshake().await.unwrap();

    let connection = connection.establish(3, 10).await;
    let rpc_manager = connection.read().await.create_rpc_manager().await;

    // TODO: Avoid hangs. maybe add a new CallID type?
    let call_id = client::Add {
        arg0: 10, arg1: 20
    }.call_dropped(&rpc_manager).await.unwrap();
    
    
    {
        let now = SystemTime::now();
        
        let res = client::AddPipelined {
            arg0: call_id.pipeline(),
            arg1: 20.into()
        }.call(&rpc_manager).await.unwrap();
        
        info!(?res, time=?now.elapsed().unwrap(), "AddPipelined");
    }
    
    let res = client::AppendString {
        arg0: "Hello ".to_string(),
        arg1: "World".to_string()
    }.call(&rpc_manager).await.unwrap();

    info!(res, "AppendString");

    let res = client::RangeVec {
        arg0: 200
    }.call(&rpc_manager).await.unwrap();

    info!(?res, "RangeVec");

    let res = client::CallMeToPanic {}.call(&rpc_manager).await.unwrap();
    info!(?res, "CallMeToPanic");

    connection.write().await.close().await.unwrap();
    info!("Sent!");
}
