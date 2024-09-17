use serde::Serialize;

use kvlr::{client::request::Request, rpc::pipelining::MaybePipelinedValue};
// TODO: Make this be imported automatically
use kvlr_derives::Request;

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1337]
#[kvlr_request_is_pipelined = false]
#[kvlr_request_response = "u32"]
pub struct Add {
    pub arg0: u32,
    pub arg1: u32,
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1337]
#[kvlr_request_is_pipelined = true]
#[kvlr_request_response = "u32"]
pub struct AddPipelined {
    pub arg0: MaybePipelinedValue<u32>,
    pub arg1: MaybePipelinedValue<u32>,
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1234]
#[kvlr_request_is_pipelined = false]
#[kvlr_request_response = "String"]
pub struct AppendString {
    pub arg0: String,
    pub arg1: String,
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1234]
#[kvlr_request_is_pipelined = true]
#[kvlr_request_response = "String"]
pub struct AppendStringPipelined {
    pub arg0: MaybePipelinedValue<String>,
    pub arg1: MaybePipelinedValue<String>,
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 4321]
#[kvlr_request_is_pipelined = false]
#[kvlr_request_response = "Vec<u32>"]
pub struct RangeVec {
    pub arg0: u32,
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 4321]
#[kvlr_request_is_pipelined = true]
#[kvlr_request_response = "Vec<u32>"]
pub struct RangeVecPipelined {
    pub arg0: MaybePipelinedValue<u32>,
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1111]
#[kvlr_request_is_pipelined = false]
#[kvlr_request_response = "()"]
pub struct CallMeToPanic;

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1111]
#[kvlr_request_is_pipelined = true]
#[kvlr_request_response = "()"]
pub struct CallMeToPanicPipelined;
