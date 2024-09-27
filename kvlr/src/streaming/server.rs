use std::{collections::HashMap, sync::Arc};

use tracing::warn;

use crate::{
    connection::Connection,
    rpc::{
        connection_state::{into_handler, HandlerFn},
        pipelining::MaybePipelinedValue,
    },
};

use super::StreamID;

pub struct StreamRpc;

impl StreamRpc {
    fn handle_incoming_stream(conn: Arc<Connection>, stream_id: StreamID, stream_data: Vec<u8>) {
        if let Some(chan) = conn.streaming_state.incoming_streams.read().unwrap().get(&stream_id) {
            let _ = chan.send(stream_data);
        } else {
            // TODO: Implement caching or auto-register streams via the "atom" system
            warn!("Warning: {:?} is not registered yet.", stream_id);
        }
    }

    pub fn register(fns_map: &mut HashMap<u32, Arc<dyn HandlerFn>>) {
        // TODO: Replace this with a macro_rules macro
        fns_map.insert(
            1,
            into_handler(move |conn, pld, slice: Vec<u8>| {
                async move {
                    // FIXME: Do we really need pipelining for streams?
                    let (stream_id, stream_data): (u32, Vec<u8>) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<u32>, MaybePipelinedValue<Vec<u8>>) =
                            rmp_serde::from_slice(&slice).unwrap();
                        (
                            args.0.resolve(&pld).await.unwrap(),
                            args.1.resolve(&pld).await.unwrap(),
                        )
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = Self::handle_incoming_stream(conn, stream_id.into(), stream_data);

                    // Always returns void
                    rmp_serde::to_vec(&out).unwrap()
                }
            }),
        );
    }
}
