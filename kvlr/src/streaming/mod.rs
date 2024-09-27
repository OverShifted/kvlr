use serde::{Deserialize, Serialize};

// TODO: Make thig pub(crate)?
pub mod client;
pub(crate) mod connection_state;
pub mod server;
pub mod stream_receiver;
pub mod stream_sender;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct StreamID(pub u32);

impl From<u32> for StreamID {
    fn from(v: u32) -> StreamID {
        StreamID(v)
    }
}
