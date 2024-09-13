use std::{collections::HashMap, sync::Arc};

use multimap::MultiMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, RwLock};

use super::{CallID, RpcResponse};

#[derive(Clone, Default)]
pub struct PipeliningData {
    // TODO: SUPER IMPORTANT: Drop values after some time!
    // TODO: Avoid encoding to byte
    available: Arc<RwLock<HashMap<CallID, RpcResponse>>>,
    wishlist: Arc<RwLock<MultiMap<CallID, oneshot::Sender<RpcResponse>>>>
}

impl PipeliningData {
    pub async fn add_result(&self, call_id: CallID, value: RpcResponse) {
        // TODO: maybe do something if we replaced an existing value?
        let _ = self.available.write().await.insert(call_id, value.clone());

        if let Some(senders) = self.wishlist.write().await.remove(&call_id) {
            for sender in senders {
                // TODO: Ignore errors?
                let _ = sender.send(value.clone());
            }
        }
    }

    pub async fn wishlist(&self, call_id: CallID, sender: oneshot::Sender<RpcResponse>) {
        if let Some(value) = self.available.read().await.get(&call_id) {
            let _ = sender.send(value.clone());
            return;
        }

        self.wishlist.write().await.insert(call_id, sender);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MaybePipelinedValue<T> {
    Direct(T),
    Pipelined(CallID)
}

impl<'a, T: serde::de::DeserializeOwned> MaybePipelinedValue<T> {
    // TODO: Better error handling
    pub async fn resolve(self, pipelining: &PipeliningData) -> Result<T, ()> {
        match self {
            MaybePipelinedValue::Direct(v) => Ok(v),
            MaybePipelinedValue::Pipelined(call_id) => {
                let (tx, rx) = oneshot::channel();

                pipelining.wishlist(call_id, tx).await;

                // TODO: Decode errors?
                let bytes = rx.await.map_err(|_| ())?.map_err(|_| ())?;
                Ok(rmp_serde::from_slice(&bytes).unwrap())
            }
        }
    }
}

impl<T> From<T> for MaybePipelinedValue<T> {
    fn from(value: T) -> Self {
        MaybePipelinedValue::Direct(value)
    }
}
