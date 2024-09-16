use std::sync::{Arc, Mutex};

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::oneshot;

use crate::{client::request::Request, connection::{frame::Frame, ConnectionFrameSender}};

use super::{connection_state::{OneshotResponseReceiver, Promises}, CallID};

// An "entry point" for calling functions
#[derive(Clone)]
pub struct RpcManager {
    pub(crate) frame_sender: ConnectionFrameSender,
    pub(crate) promises: Promises,

    pub(crate) next_call_id: Arc<Mutex<u32>>,
}

impl RpcManager {
    pub async fn call_request_dropped<T: Request>(&self, request: &T) -> Result<CallID, ()> {
        let out = self.call_raw(
            T::FUNCTION_ID,
            T::IS_PIPELINED,
            false,
            rmp_serde::to_vec(request).unwrap()
        ).await;

        match out {
            // TODO: Make call_raw return the called call_id
            Ok(_rx) => Ok((*self.next_call_id.lock().unwrap() - 1).into()),
            Err(e) => Err(e)
        }
    }

    pub async fn call_request<T: Request>(&self, request: &T) -> Result<T::Response, ()> {
        let out = self.call_raw(
            T::FUNCTION_ID,
            T::IS_PIPELINED,
            false,
            rmp_serde::to_vec(request).unwrap()
        ).await;

        let respnse_wire = out.map_err(|_| ())?.await.map_err(|_| ())?.map_err(|_| ())?;
        Ok(rmp_serde::from_slice(&respnse_wire).unwrap())
    }

    pub async fn call_dropped<Args: Serialize>(
        &self,
        function_id: u32,
        is_pipelined: bool,
        args: &Args,
    ) -> Result<CallID, ()> {
        let out = self
            .call_raw(
                function_id,
                is_pipelined,
                true,
                rmp_serde::to_vec(args).unwrap(),
            )
            .await;

        match out {
            // TODO: Make call_raw return the called call_id
            Ok(_) => Ok((*self.next_call_id.lock().unwrap() - 1).into()),
            Err(e) => Err(e)
        }
    }

    // TODO: Better errors
    pub async fn call<Args: Serialize, Out: DeserializeOwned>(
        &self,
        function_id: u32,
        is_pipelined: bool,
        args: &Args,
    ) -> Result<Out, ()> {
        let out = self
            .call_raw(
                function_id,
                is_pipelined,
                false,
                rmp_serde::to_vec(args).unwrap(),
            )
            .await;

        let respnse_wire = out.map_err(|_| ())?.await.map_err(|_| ())?.map_err(|_| ())?;
        Ok(rmp_serde::from_slice(&respnse_wire).unwrap())
    }

    // TODO: Better error handling
    // TODO: Hangs on server panic
    pub async fn call_raw(
        &self,
        function_id: u32,
        is_pipelined: bool,
        drop_answer: bool,
        args: Vec<u8>,
    ) -> Result<OneshotResponseReceiver, ()> {
        // info!("{}", is_pipelined);
        let call_id = self.get_next_call_id().await;

        let flags: u8 =
            0b1 | if is_pipelined { 0b10 } else { 0b0 } | if drop_answer { 0b100 } else { 0b0 };

        // info!(call_id, is_pipelined, "{:b}", flags);

        let body = vec![
            vec![flags],
            function_id.to_be_bytes().to_vec(),
            call_id.0.to_be_bytes().to_vec(),
            args,
        ]
        .concat();

        // TODO: Avoid createing channel if dropping answer
        let (tx, rx) = oneshot::channel();

        if !drop_answer {
            self.promises.register(call_id, tx).await;
        }

        self.frame_sender
            .send_frame(Frame {
                protocol: "rpc".to_string(),
                body,
            })
            .await
            .unwrap();

        Ok(rx)
    }

    async fn get_next_call_id(&self) -> CallID {
        let mut lock = self.next_call_id.lock().unwrap();
        let prev_value = *lock;
        *lock += 1;

        CallID(prev_value)
    }
}
