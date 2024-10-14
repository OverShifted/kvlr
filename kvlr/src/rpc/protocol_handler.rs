use std::{
    io::{Cursor, Read},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Buf;
use tracing::{error, warn};

use crate::{
    connection::{
        frame::Frame, protocol_handler::ProtocolHandler, Connection, ConnectionFrameSender,
    },
    rpc::InternalServerError,
};

use super::{
    connection_state::{Functions, Promises},
    pipelining::PipeliningData,
    CallID,
};

pub struct RpcProtocolHandler;

struct HandleCallParams<'a> {
    frame: &'a Frame,
    is_pipelined: bool,
    drop_answer: bool,
    functions: &'a Functions,
    frame_sender: &'a ConnectionFrameSender,
    pipelining_data: &'a PipeliningData,
}

impl RpcProtocolHandler {
    async fn handle_call(
        &self,
        connection: &Arc<Connection>,
        mut reader: impl Buf,
        params: HandleCallParams<'_>,
    ) {
        let fn_id = reader.get_u32();
        let call_id = CallID(reader.get_u32());
        // trace!(fn_id, ?call_id, "Incoming call");

        // TODO: Remove this lock
        let handler = {
            let functions = params.functions.0.read().unwrap();
            functions.get(&fn_id).cloned()
        };
        // info!(call_id, "Lock kardam");

        match handler {
            Some(h) => {
                // info!(call_id, "Berim handle");

                let frame_sender = params.frame_sender.clone();
                let connection = connection.clone();
                let pipelining_data = params.pipelining_data.clone();
                let args_wire = params.frame.body[9..].to_vec();

                // info!(call_id, "Berim spawn");
                tokio::spawn(async move {
                    // info!(call_id, "Salam spawn");
                    let logic_handler = {
                        let pipelining_data = if params.is_pipelined {
                            Some(pipelining_data.clone())
                        } else {
                            None
                        };

                        tokio::spawn(h(connection, pipelining_data, args_wire)).await
                    };

                    let rpc_response = match logic_handler {
                        Ok(out) => Ok(out),
                        Err(err) => {
                            error!(
                                ?err,
                                ?call_id,
                                fn_id,
                                "Call handler failed! (Panicked or canceled)"
                            );
                            Err(InternalServerError)
                        }
                    };

                    let response_wire = rmp_serde::to_vec(&rpc_response).unwrap();
                    pipelining_data.add_result(call_id, rpc_response).await;

                    if !params.drop_answer {
                        let body =
                            [vec![0u8], call_id.0.to_be_bytes().to_vec(), response_wire].concat();

                        frame_sender
                            .send_frame(Frame {
                                protocol: "rpc".to_string(),
                                body,
                            })
                            .await
                            .unwrap();
                    }
                    // info!(call_id, "Resolved call");
                }).await.unwrap();
            }

            None => error!(?call_id, fn_id, "No RPC function handler for"),
        }
    }

    async fn handle_response(&self, mut reader: impl Buf + Read, promises: &Promises) {
        let call_id = CallID(reader.get_u32());

        {
            let handler = {
                let mut promises = promises.0.write().unwrap();
                promises.remove(&call_id)
            };

            match handler {
                Some(h) => {
                    let _ = h.send(rmp_serde::from_read(&mut reader).unwrap());
                }

                None => warn!(?call_id, "No response handler registered for"),
            }
        }
    }
}

#[async_trait]
impl ProtocolHandler for RpcProtocolHandler {
    async fn handle_frame(&self, connection: &Arc<Connection>, frame: &Frame) {
        let functions = connection.rpc_state.functions.clone();
        let promises = connection.rpc_state.promises.clone();

        let mut reader = Cursor::new(&frame.body);
        
        let flags = reader.get_u8();
        let is_call = flags & 0b1 != 0;
        let is_pipelined = flags & 0b10 != 0;
        let drop_answer = flags & 0b100 != 0;

        if is_call {
            self.handle_call(
                connection,
                reader,
                HandleCallParams {
                    frame,
                    is_pipelined,
                    drop_answer,
                    functions: &functions,

                    // TODO: This call locks
                    frame_sender: &connection.create_frame_sender().await,
                    pipelining_data: &connection.rpc_state.pipelining_data.clone(),
                },
            )
            .await;
        } else {
            self.handle_response(reader, &promises).await;
        };
    }
}
