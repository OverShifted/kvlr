use serde::{Deserialize, Serialize};

// TODO: Make thig pub(crate)?
pub mod server;
pub mod client;
pub mod stream_sender;
pub mod stream_receiver;
pub(crate) mod connection_state;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct StreamID(pub u32);

impl From<u32> for StreamID {
    fn from(v: u32) -> StreamID {
        StreamID(v)
    }
}
