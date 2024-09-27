use serde::Serialize;

use crate::{client::request::Request, rpc::pipelining::MaybePipelinedValue};
// TODO: Make this be imported automatically
use kvlr_derives::Request;

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1]
#[kvlr_request_is_pipelined = false]
#[kvlr_request_response = "()"]
pub struct Stream {
    pub arg0: u32,
    pub arg1: Vec<u8>
}

#[derive(Debug, Serialize, Request)]
#[kvlr_request_function_id = 1]
#[kvlr_request_is_pipelined = true]
#[kvlr_request_response = "()"]
pub struct StreamPipelined {
    pub arg0: MaybePipelinedValue<u32>,
    pub arg1: MaybePipelinedValue<Vec<u8>>
}
