use serde::{de::DeserializeOwned, Serialize};

use crate::{promise_utils::FutureSyncSend, rpc::{rpc_manager::RpcManager, CallID}};

pub trait Request: Serialize + Sized + Sync {
    const FUNCTION_ID: u32;
    const IS_PIPELINED: bool;

    type Response: DeserializeOwned;

    fn call(&self, rpc_manager: RpcManager) -> impl FutureSyncSend<Result<Self::Response, ()>> {
        async move {
            rpc_manager.call_request(self).await
        }
    }

    fn call_dropped(&self, rpc_manager: RpcManager) -> impl FutureSyncSend<Result<CallID, ()>> {
        async move {
            rpc_manager.call_request_dropped(self).await
        }
    }
}
