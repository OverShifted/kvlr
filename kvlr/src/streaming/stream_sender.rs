use std::marker::PhantomData;

use serde::Serialize;

use crate::{client::request::Request, rpc::rpc_manager::RpcManager};

use super::{client::Stream, StreamID};

pub struct StreamSender<T: Serialize> {
    stream_id: StreamID,
    rpc_manager: RpcManager,
    pd: PhantomData<T>
}

impl<T: Serialize> StreamSender<T> {
    pub fn new(stream_id: StreamID, rpc_manager: RpcManager) -> StreamSender<T> {
        StreamSender {
            stream_id,
            rpc_manager,
            pd: PhantomData
        }
    }

    pub async fn send_and_ack(&self, items: &[T]) -> Result<(), ()> {
        // TODO: Maybe we can prevent RpcManager from getting cloned?
        self.as_stream_request(items).call(self.rpc_manager.clone()).await
    }

    pub async fn send(&self, items: &[T]) -> Result<(), ()> {
        // TODO: Maybe we can prevent RpcManager from getting cloned?
        self.as_stream_request(items).call_dropped(self.rpc_manager.clone()).await.map(|_| ())
    }

    fn as_stream_request(&self, items: &[T]) -> Stream {
        Stream {
            arg0: self.stream_id.0,

            // TODO: Prevent copies
            arg1: rmp_serde::to_vec(&items).unwrap()
        }
    }
}
