use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tracing::info;

use crate::connection::Connection;

use super::StreamID;

pub struct StreamReceiver<T: DeserializeOwned> {
    // TODO: Remove pub?
    pub rx: mpsc::Receiver<Vec<u8>>,
    pd: PhantomData<T>,
}

impl<T: DeserializeOwned> StreamReceiver<T> {
    /// Creates a new StreamReceiver and registers it to the connection
    pub fn new(id: StreamID, connection: &Connection, capacity: usize) -> StreamReceiver<T> {
        let (tx, rx) = mpsc::channel(capacity);
        connection
            .streaming_state
            .incoming_streams
            .write()
            .unwrap()
            .insert(id, tx);

        info!(?id, "Registered stream recv");
        StreamReceiver {
            rx,
            pd: PhantomData,
        }
    }
}
