pub mod frame;
mod processor;
pub(crate) mod protocol_handler;
pub mod stream;

use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, RwLock};
use std::{collections::HashMap, mem, sync::Arc};

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    task::JoinHandle,
};

use frame::Frame;
use processor::{read_processor, write_processor};
use stream::Stream;
use tracing::info;

use crate::rpc::{
    self,
    connection_state::{Functions, HandlerFn, Promises},
    rpc_manager::RpcManager,
};
use crate::streaming;

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("Error while reading handshake")]
    ReadError,
    #[error("Invalid data recived while waiting for handshake")]
    InvalidData,
    #[error("Error while writing handshake")]
    WriteError,
}

pub trait StreamRead: AsyncRead + Send + Sync + Unpin {}
impl<T> StreamRead for T where T: AsyncRead + Send + Sync + Unpin {}

pub trait StreamWrite: AsyncWrite + Send + Sync + Unpin {}
impl<T> StreamWrite for T where T: AsyncWrite + Send + Sync + Unpin {}

enum State {
    PreHandshake {
        read: Box<dyn StreamRead>,
        write: Box<dyn StreamWrite>,
    },

    Established {
        // TODO: Remove this
        _rx: Receiver<Frame>,
        tx: Sender<(Frame, oneshot::Sender<std::io::Result<()>>)>,

        read_processor_handle: Option<JoinHandle<()>>,
        write_processor_handle: Option<JoinHandle<()>>,
    },

    // Not the best idea ever!
    Undefined,
}

impl State {
    fn get_read(&mut self) -> &mut Box<dyn StreamRead> {
        match self {
            State::PreHandshake { ref mut read, .. } => read,
            _ => panic!("Trying to directly access read handle of a kvlr::Connection!"),
        }
    }

    fn get_write(&mut self) -> &mut Box<dyn StreamWrite> {
        match self {
            State::PreHandshake { ref mut write, .. } => write,
            _ => panic!("Trying to directly access write handle of a kvlr::Connection!"),
        }
    }
}

pub struct Connection {
    // TODO: Make this a normal lock
    state: tokio::sync::RwLock<State>,

    // Used to store connection-specific information for RPC
    pub(crate) rpc_state: rpc::connection_state::ConnectionState,
    pub(crate) streaming_state: streaming::connection_state::ConnectionState,
}

impl Connection {
    pub fn new<T: Stream + Sync + 'static>(
        stream: T,
        functions: Arc<RwLock<HashMap<u32, Arc<dyn HandlerFn>>>>,
    ) -> Connection {
        let (stream_read, stream_write) = tokio::io::split(stream);

        Connection {
            state: tokio::sync::RwLock::new(State::PreHandshake {
                read: Box::new(stream_read),
                write: Box::new(stream_write),
            }),

            rpc_state: rpc::connection_state::ConnectionState {
                functions: Functions(functions),
                pipelining_data: Default::default(),

                promises: Default::default(),
                // It starts with one...
                next_call_id: Arc::new(Mutex::new(1)),
            },

            streaming_state: streaming::connection_state::ConnectionState {
                incoming_streams: RwLock::new(Default::default()),
            },
        }
    }

    // TODO: We might find the "error" early in buf, so we might not need to read it all.
    async fn wait_for_data(&self, data: &[u8]) -> Result<(), HandshakeError> {
        let mut buf = vec![0; data.len()];
        self.state
            .write()
            .await
            .get_read()
            .read_exact(&mut buf)
            .await
            .map_err(|_| HandshakeError::ReadError)?;

        if data == buf {
            Ok(())
        } else {
            Err(HandshakeError::InvalidData)
        }
    }

    /// Performs server's role in handshake
    pub async fn recv_handshake(&self) -> Result<(), HandshakeError> {
        self.wait_for_data(b"KVLR").await?;
        self.state
            .write()
            .await
            .get_write()
            .write_all(b"KVLR")
            .await
            .map_err(|_| HandshakeError::WriteError)?;
        Ok(())
    }

    /// Performs client's role in handshake
    pub async fn send_handshake(&self) -> Result<(), HandshakeError> {
        self.state
            .write()
            .await
            .get_write()
            .write_all(b"KVLR")
            .await
            .map_err(|_| HandshakeError::WriteError)?;
        self.wait_for_data(b"KVLR").await?;
        Ok(())
    }

    pub async fn send_frame(&self, frame: Frame) -> std::io::Result<()> {
        self.create_frame_sender().await.send_frame(frame).await
    }

    /// Changes the connection's state to established.
    /// MUST be called manually after handshaking
    pub async fn establish(self, rx_buffer_size: usize, tx_buffer_size: usize) -> Arc<Connection> {
        // Not the best thing we could do...
        // FIXME:
        let prev_state = {
            let mut lock = self.state.write().await;
            mem::replace(&mut (*lock), State::Undefined)
        };

        let (stream_read, stream_write) = match prev_state {
            State::PreHandshake { read, write } => (read, write),
            _ => panic!(
                "Attempting to establish a connection which is not in the pre-handshake state."
            ),
        };

        let (tx, out_rx) = mpsc::channel(rx_buffer_size);
        let (out_tx, rx) = mpsc::channel(tx_buffer_size);

        let self_arc = Arc::new(self);

        // TODO: processors might depend on State::Established?
        *self_arc.state.write().await = State::Established {
            _rx: out_rx,
            tx: out_tx,

            read_processor_handle: None,
            write_processor_handle: None,
        };

        let read_processor_handle_right = tokio::spawn(read_processor(self_arc.clone(), stream_read, tx));
        let write_processor_handle_right = tokio::spawn(write_processor(stream_write, rx));

        {
            let mut lock = self_arc.state.write().await;
            if let State::Established {
                read_processor_handle,
                write_processor_handle,
                ..
            } = lock.deref_mut()
            {
                *read_processor_handle = Some(read_processor_handle_right);
                *write_processor_handle = Some(write_processor_handle_right);
            }
        }

        self_arc
    }

    // TODO: Colse the underlying connection somewhere
    pub async fn close(&self) -> std::io::Result<()> {
        self.send_frame(Frame {
            protocol: "close".into(),
            body: vec![],
        })
        .await?;

        {
            let mut lock = self.state.write().await;
            if let State::Established {
                read_processor_handle,
                write_processor_handle,
                ..
            } = lock.deref()
            {
                info!("Aborting processors");
                read_processor_handle.as_ref().unwrap().abort();
                write_processor_handle.as_ref().unwrap().abort();
            }

            // TODO: Maybe add a new disconnected state?
            // TODO: It (at least used to) sometimes panicked
            *lock = State::Undefined;
        }

        Ok(())
    }

    pub async fn get_functions_ref(&self) -> Functions {
        self.rpc_state.functions.clone()
    }

    pub async fn get_promises_ref(&self) -> Promises {
        self.rpc_state.promises.clone()
    }

    pub(crate) async fn create_frame_sender(&self) -> ConnectionFrameSender {
        match *self.state.read().await {
            State::Established { ref tx, .. } => ConnectionFrameSender(tx.clone()),
            _ => panic!(
                "Attempting to create a frame sender on a connection which is not established"
            ),
        }
    }

    pub async fn create_rpc_manager(&self) -> RpcManager {
        RpcManager {
            frame_sender: self.create_frame_sender().await,
            promises: self.get_promises_ref().await,
            next_call_id: self.rpc_state.next_call_id.clone(),
        }
    }
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}

#[derive(Clone)]
pub(crate) struct ConnectionFrameSender(Sender<(Frame, oneshot::Sender<std::io::Result<()>>)>);

impl ConnectionFrameSender {
    pub(crate) async fn send_frame(&self, frame: Frame) -> std::io::Result<()> {
        let (ret_tx, ret_rx) = oneshot::channel();

        // TODO: Is the unwrap always "valid"?
        self.0.send((frame, ret_tx)).await.unwrap();

        // TODO: Is the unwrap always "valid"?
        ret_rx.await.unwrap()
    }
}
