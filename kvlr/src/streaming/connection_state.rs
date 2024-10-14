use std::{collections::HashMap, sync::RwLock};

use tokio::sync::mpsc;

use super::StreamID;

pub(crate) struct ConnectionState {
    pub(crate) incoming_streams: RwLock<HashMap<StreamID, mpsc::Sender<Vec<u8>>>>,
}
