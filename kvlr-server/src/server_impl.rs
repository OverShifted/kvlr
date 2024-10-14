use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use diesel::{prelude::*, Connection};

#[allow(unused_imports)]
use kvlr::{
    connection::Connection as KConn,
    streaming::{stream_receiver::StreamReceiver, stream_sender::StreamSender, StreamID},
};
use sha2::Digest;
use tracing::{info, trace};

use crate::server_trait::SomeFunctions;

pub struct ServerImpl {
    db: Mutex<PgConnection>,
}

fn hash_string_to_hex(string: &str) -> String {
    let mut hasher: _ = sha2::Sha256::new();
    hasher.update(string);
    let hash = hasher.finalize();
    hex::encode(&hash)
}

impl ServerImpl {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mut db = PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        info!("Connected to database");

        use super::models::*;
        use super::schema::users::dsl::*;

        // diesel::insert_into(users)
        //     .values(NewUser {
        //         username: "thrd2",
        //         secret_sha256: &hash_string_to_hex("supersecret"),
        //     })
        //     .execute(&mut db)
        //     .unwrap();

        let results = users.select(User::as_select()).load(&mut db).unwrap();
        for user in results {
            trace!("{:?}", user);
        }

        ServerImpl { db: Mutex::new(db) }
    }
}

#[async_trait]
impl SomeFunctions for ServerImpl {
    async fn login(&self, _conn: Arc<KConn>, in_username: String, password: String) -> bool {
        // self.db.lock()
        use super::models::*;
        use super::schema::users::dsl::*;

        let result = users
            .filter(username.eq(in_username))
            .filter(secret_sha256.eq(hash_string_to_hex(&password)))
            .select(User::as_select())
            .get_result(&mut *self.db.lock().unwrap());

        info!("{:?}", result);

        result.is_ok()
    }

    // async fn upload_file(&self, connection: Arc<KConn>, stream_id: u32) -> () {
    //     let mut rx = StreamReceiver::<u8>::new(stream_id.into(), &connection, 1);
    //     tokio::spawn(async move {
    //         let mut file = std::fs::File::create("received-file.bin").unwrap();

    //         while let Some(items_bytes) = rx.rx.recv().await {
    //             let items: Vec<u8> = rmp_serde::from_slice(&items_bytes).unwrap();
    //             trace!(items.len = items.len(), "Recved");
    //             file.write_all(&items).unwrap();
    //         }

    //         file.flush().unwrap();
    //     });

    //     ()
    // }

    async fn add(&self, _conn: Arc<KConn>, arg0: u32, arg1: u32) -> u32 {
        arg0 + arg1
    }

    async fn append_string(&self, _conn: Arc<KConn>, arg0: String, arg1: String) -> String {
        arg0 + &arg1
    }

    // async fn range_vec(&self, conn: Arc<KConn>, arg0: u32) -> Vec<u32> {
    //     tokio::spawn(async move {
    //         let sender: StreamSender<String> =
    //             StreamSender::new(StreamID(42), conn.create_rpc_manager().await);
    //         let items: Vec<String> = vec![
    //             "I".into(),
    //             " need".into(),
    //             " a".into(),
    //             " beqe".into(),
    //             " beqe".into(),
    //             " a".into(),
    //             " beqe".into(),
    //             " is".into(),
    //             " all".into(),
    //             " I".into(),
    //             " need".into(),
    //             ".".into(),
    //         ];

    //         for chunk in items.chunks(2) {
    //             sender.send_and_ack(chunk.into()).await.unwrap();
    //             // tokio::time::sleep(Duration::from_millis(500)).await;
    //         }
    //     });

    //     (0..arg0).collect()
    // }

    // async fn call_me_to_panic(&self, _conn: Arc<KConn>) -> () {
    //     panic!("I was intended to panic!")
    // }
}
