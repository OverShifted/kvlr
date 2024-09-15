use std::{collections::HashMap, pin::Pin, sync::Arc};

use tokio::sync::{oneshot, Mutex, RwLock};

use crate::promise_utils::FutureSyncSend;
use super::{pipelining::PipeliningData, CallID, RpcResponse};

pub trait HandlerFn:
    (Fn(Option<PipeliningData>, Vec<u8>) -> Pin<Box<dyn FutureSyncSend<Vec<u8>>>>) + Sync + Send
{
}

impl<T> HandlerFn for T where
    T: (Fn(Option<PipeliningData>, Vec<u8>) -> Pin<Box<dyn FutureSyncSend<Vec<u8>>>>) + Sync + Send
{
}

// Turns an async closure into a proper HandlerFn
pub fn into_handler<F, T>(fnc: F) -> Arc<dyn HandlerFn>
where
    F: 'static + Sync + Send + Fn(Option<PipeliningData>, Vec<u8>) -> T,
    T: 'static + FutureSyncSend<Vec<u8>>,
{
    Arc::new(move |pld, slice| Box::pin(fnc(pld, slice)))
}

pub(crate) struct ConnectionState {
    // Server-side
    // fn_id -> handler
    pub(crate) functions: Functions,
    pub(crate) pipelining_data: PipeliningData,

    // Client-side
    // TODO: Limit max panding promises?
    pub(crate) promises: Promises,
    pub(crate) next_call_id: Arc<Mutex<u32>>,
}

// A [reference to a] set of rpc functions handlers
#[derive(Clone, Default)]
pub struct Functions(pub Arc<RwLock<HashMap<u32, Arc<dyn HandlerFn>>>>);

impl Functions {
    /// Registers a new function handler.
    /// If another handler for the same function already exists, it is returned.
    // TODO: Add a batch version to avoid locking so many times
    pub async fn register(
        &self,
        function_id: u32,
        handler: Arc<dyn HandlerFn>,
    ) -> Option<Arc<dyn HandlerFn>> {
        self.0.write().await.insert(function_id, handler)
    }
}

pub type OneshotResponseSender = oneshot::Sender<RpcResponse>;
pub type OneshotResponseReceiver = oneshot::Receiver<RpcResponse>;

// A [reference to a] set of rpc pending promises
#[derive(Clone, Default)]
pub struct Promises(pub Arc<RwLock<HashMap<CallID, OneshotResponseSender>>>);

impl Promises {
    pub async fn register(
        &self,
        call_id: CallID,
        sender: OneshotResponseSender,
    ) -> Option<OneshotResponseSender> {
        self.0.write().await.insert(call_id, sender)
    }
}
