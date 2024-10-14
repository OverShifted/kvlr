use bytes::{BufMut, Bytes};

use crate::traits::{Deserialize, Serialize};

impl Serialize for () {
    fn serialize(&self, buf: &mut impl BufMut) {
    }
}

impl Deserialize for () {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
        Ok(())
    }
}