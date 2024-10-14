mod client;
mod repl;

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, RwLock}, time::{Duration, SystemTime},
};

use human_format::Formatter;
use rustls::{
    client::ClientConfig,
    pki_types::{pem::PemObject, CertificateDer, ServerName},
    RootCertStore,
};

use tokio::{io::{AsyncReadExt, BufReader}, net::TcpStream};
use tokio_rustls::TlsConnector;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[allow(unused_imports)]
use kvlr::{
    client::request::Request,
    connection::Connection,
    promise_utils::PromiseHelper,
    streaming::{server::StreamRpc, stream_receiver::StreamReceiver, stream_sender::StreamSender},
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

    repl::start(&rpc_manager).await;
    connection.close().await.unwrap();
    return;

    let resp = client::Add { arg0: 10, arg1: 20 }
        .call(rpc_manager.clone())
        .await
        .unwrap();

    info!("add(10, 20) -> {resp}");

    connection.close().await.unwrap();

    // TODO: Avoid hangs. maybe add a new CallID type?
    // let call_id = client::Add { arg0: 10, arg1: 20 }
    //     .call_dropped(rpc_manager.clone())
    //     .await
    //     .unwrap();

    // {
    //     let now = SystemTime::now();

    //     let res = client::AddPipelined {
    //         arg0: call_id.pipeline(),
    //         arg1: 20.into(),
    //     }
    //     .call(rpc_manager.clone())
    //     .await
    //     .unwrap();

    //     info!(?res, time=?now.elapsed().unwrap(), "AddPipelined");
    // }

    // let res = client::AppendString {
    //     arg0: "Hello ".to_string(),
    //     arg1: "World".to_string(),
    // }
    // .call(rpc_manager.clone())
    // .await
    // .unwrap();

    // info!(res, "AppendString");

    // let mut recv = StreamReceiver::<String>::new(42.into(), &connection, 10);
    // tokio::spawn(async move {
    //     while let Ok(items_bytes) = recv.rx.recv().await {
    //         let items: Vec<String> = rmp_serde::from_slice(&items_bytes).unwrap();
    //         info!(?items, "Item(s) recved!");
    //     }
    // });

    // let res = client::RangeVec { arg0: 200 }
    //     .call(rpc_manager.clone())
    //     .await
    //     .unwrap();

    // info!(?res, "RangeVec");

    // client::CallMeToPanic.call(rpc_manager).on(
    //     |s| async move {
    //         info!(?s, "Yay!");
    //     },
    //     |e| async move {
    //         error!(?e, "Opps! Server returned an error!");
    //     },
    // );

    // tokio::time::sleep(Duration::from_millis(1000 * 5)).await;

    // client::UploadFile { arg0: 42 }
    //     .call(rpc_manager.clone())
    //     .await
    //     .unwrap();
    // let tx = StreamSender::<u8>::new(42.into(), rpc_manager);

    // TODO: This doesn't seem to actually use efficent storage and wont even be decoded correctly
    // use serde::ser::Serialize;
    // let mut msgpack_data = Vec::new();
    // let mut serializer = rmp_serde::Serializer::new(&mut msgpack_data)
    //     .with_bytes(rmp_serde::config::BytesMode::ForceAll);
    // let input = "Hello world!".as_bytes();
    // // let input = &[0, 1, 2, 3, 230];
    // input.serialize(&mut serializer).unwrap();
    // // tx.send(&rmp_serde::to_vec(&input).unwrap()).await.unwrap();
    // let back: &[u8] = rmp_serde::from_slice(&msgpack_data).unwrap();
    // tx.send(&input).await.unwrap();

    // info!(?input, "");
    // info!(back_ = ?back, "");
    // info!(?msgpack_data, "");
    // info!(msgpack_norm = ?rmp_serde::to_vec(&input).unwrap(), "");

    // {
    //     let file = tokio::fs::File::open("sample-file").await.unwrap();
    //     let mut file = BufReader::new(file);
        
    //     let mut buffer = [0u8; 1024 * 5];
    //     // let mut i = 0;
    //     loop {
            
    //         let n = match file.read(&mut buffer).await {
    //             Ok(0) => break,
    //             Ok(n) => n,
    //             Err(_) => break,
    //         };

    //         let now = SystemTime::now();
            
    //         tx.send(&buffer[..n]).await.unwrap();

    //         let duration = now.elapsed().unwrap();
    //         let speed = n as f64 / duration.as_secs_f64();

    //         let speed_human = Formatter::new()
    //             .with_units("B/s")
    //             .format(speed);

    //         info!(?duration, speed=%speed_human, "Sent {n} bytes");
    //         // i += 1;
    //     }

    //     tokio::time::sleep(Duration::from_millis(2)).await;
    //     connection.close().await.unwrap();
    //     info!("Sent!");
    // }
}
