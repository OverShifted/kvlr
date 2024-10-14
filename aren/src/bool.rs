use bytes::{BufMut, Bytes};

use crate::traits::{Deserialize, Serialize};

impl Serialize for bool {
    fn serialize(&self, buf: &mut impl BufMut) {
        (*self as u8).serialize(buf);
    }
}

impl Deserialize for bool {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
        u8::deserialize(buf).map(|i| i != 0)
    }
}