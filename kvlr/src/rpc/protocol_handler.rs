use std::{io::{Cursor, Read}, sync::Arc};

use async_trait::async_trait;
use bytes::Buf;
use tokio::sync::RwLock;
use tracing::{error, warn};

use crate::{connection::{frame::Frame, protocol_handler::ProtocolHandler, Connection, ConnectionFrameSender}, rpc::InternalServerError};

use super::{connection_state::{Functions, Promises}, pipelining::PipeliningData, CallID};

pub struct RpcProtocolHandler;

impl RpcProtocolHandler {
    async fn handle_call(&self, mut reader: impl Buf, frame: &Frame, is_pipelined: bool, drop_answer: bool, functions: &Functions, frame_sender: &ConnectionFrameSender, pipelining_data: &PipeliningData) {
        let fn_id = reader.get_u32();
        let call_id = CallID(reader.get_u32());
        // info!(call_id, is_pipelined, "Incoming call");

        // TODO: Remove this lock
        let handler = {
            let functions = functions.0.read().unwrap();
            functions.get(&fn_id).map(|h| h.clone())
        };
        // info!(call_id, "Lock kardam");

        match handler {
            Some(h) => {
                // info!(call_id, "Berim handle");

                let args_wire = frame.body[9..].to_vec();
                let frame_sender = frame_sender.clone();
                let pipelining_data = pipelining_data.clone();

                // info!(call_id, "Berim spawn");
                tokio::spawn(async move {
                    // info!(call_id, "Salam spawn");
                    let logic_handler = {
                        let pipelining_data = if is_pipelined {
                            Some(pipelining_data.clone())
                        } else {
                            None
                        };

                        tokio::spawn(h(pipelining_data,args_wire)).await
                    };

                    let rpc_response = match logic_handler {
                        Ok(out) => {
                            Ok(out)
                        }
                        Err(err) => {
                            error!(?err, ?call_id, fn_id, "Call handler failed! (Probably panicked)");
                            Err(InternalServerError)
                        }
                    };

                    let response_wire = rmp_serde::to_vec(&rpc_response).unwrap();
                    pipelining_data.add_result(call_id, rpc_response).await;

                    if !drop_answer {
                        let body =
                            vec![vec![0u8], call_id.0.to_be_bytes().to_vec(), response_wire].concat();

                        frame_sender
                            .send_frame(Frame {
                                protocol: "rpc".to_string(),
                                body,
                            })
                            .await
                            .unwrap();
                    }
                    // info!(call_id, "Resolved call");
                });
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
    async fn handle_frame(&self, connection: &Arc<RwLock<Connection>>, frame: &Frame) {
        let (functions, promises, frame_sender, pipelining_data) = {
            let connection = connection.read().await;

            let functions = connection.rpc_state.functions.clone();
            let promises = connection.rpc_state.promises.clone();
            let frame_sender = connection.create_frame_sender().await;
            let pipelining_data = connection.rpc_state.pipelining_data.clone();

            (functions, promises, frame_sender, pipelining_data)
        };

        let mut reader = Cursor::new(&frame.body);

        let flags = reader.get_u8();
        let is_call = flags & 0b1 != 0;
        let is_pipelined = flags & 0b10 != 0;
        let drop_answer = flags & 0b100 != 0;

        if is_call {
            self.handle_call(reader, frame, is_pipelined, drop_answer, &functions, &frame_sender, &pipelining_data).await;
        } else {
            self.handle_response(reader, &promises).await;
        };
    }
}
