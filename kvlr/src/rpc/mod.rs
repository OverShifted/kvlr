pub mod connection_state;
pub mod pipelining;
pub mod protocol_handler;
pub mod rpc_manager;

use pipelining::MaybePipelinedValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CallID(pub u32);

impl CallID {
    pub fn pipeline<T>(self) -> MaybePipelinedValue<T> {
        MaybePipelinedValue::Pipelined(self)
    }
}

impl From<u32> for CallID {
    fn from(v: u32) -> CallID {
        CallID(v)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct InternalServerError;
type RpcResponse = Result<Vec<u8>, InternalServerError>;
