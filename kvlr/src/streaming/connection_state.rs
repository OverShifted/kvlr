use std::{collections::HashMap, sync::RwLock};

use tokio::sync::broadcast;

use super::StreamID;

pub(crate) struct ConnectionState {
    pub(crate) incoming_streams: RwLock<HashMap<StreamID, broadcast::Sender<Vec<u8>>>>
}
