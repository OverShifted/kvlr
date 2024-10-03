mod client;

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use rustls::{
    client::ClientConfig,
    pki_types::{pem::PemObject, CertificateDer, ServerName},
    RootCertStore,
};

use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use kvlr::{
    client::request::Request,
    connection::Connection,
    promise_utils::PromiseHelper,
    streaming::{server::StreamRpc, stream_receiver::StreamReceiver},
};

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let stream = TcpStream::connect("127.0.0.1:5857").await.unwrap();

    let ca_cert_bytes = include_bytes!("../../keys/ca.crt");
    let ca_cert = CertificateDer::pem_reader_iter(Cursor::new(ca_cert_bytes))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add(ca_cert).unwrap();

    stream.set_nodelay(true).unwrap();
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let config = TlsConnector::from(Arc::new(config));
    let domain = ServerName::try_from("localhost").unwrap();
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
