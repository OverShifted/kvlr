use std::sync::Arc;

use async_trait::async_trait;
use kvlr::{
    connection::Connection,
    streaming::{stream_sender::StreamSender, StreamID},
};

use crate::server_trait::SomeFunctions;

pub struct ServerImpl;

#[async_trait]
impl SomeFunctions for ServerImpl {
    async fn add(&self, _conn: Arc<Connection>, arg0: u32, arg1: u32) -> u32 {
        arg0 + arg1
    }

    async fn append_string(&self, _conn: Arc<Connection>, arg0: String, arg1: String) -> String {
        arg0 + &arg1
    }

    async fn range_vec(&self, conn: Arc<Connection>, arg0: u32) -> Vec<u32> {
        tokio::spawn(async move {
            let sender: StreamSender<String> =
                StreamSender::new(StreamID(42), conn.create_rpc_manager().await);
            let items: Vec<String> = vec![
                "I".into(),
                " need".into(),
                " a".into(),
                " beqe".into(),
                " beqe".into(),
                " a".into(),
                " beqe".into(),
                " is".into(),
                " all".into(),
                " I".into(),
                " need".into(),
                ".".into(),
            ];

            for chunk in items.chunks(2) {
                sender.send_and_ack(chunk.into()).await.unwrap();
                // tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        (0..arg0).collect()
    }

    async fn call_me_to_panic(&self, _conn: Arc<Connection>) -> () {
        panic!("I was intended to panic!")
    }
}
