// Is it the right place to pub use?
pub use bytes::{Buf, BufMut, Bytes};

pub trait Serialize {
    // Panics on failure
    fn serialize(&self, buf: &mut impl BufMut);
}

pub trait Deserialize: Sized {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()>;
}
