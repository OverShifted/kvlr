pub mod connection_state;
pub mod pipelining;
pub mod rpc_manager;
pub mod protocol_handler;

use pipelining::MaybePipelinedValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CallID(pub u32);

impl CallID {
    pub fn pipeline<T>(self) -> MaybePipelinedValue<T> {
        MaybePipelinedValue::Pipelined(self)
    }
}

impl Into<u32> for CallID {
    fn into(self) -> u32 { self.0 }
}

impl Into<CallID> for u32 {
    fn into(self) -> CallID { CallID(self) }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct InternalServerError;
type RpcResponse = Result<Vec<u8>, InternalServerError>;
