use bytes::{Buf, BufMut, Bytes};

use crate::{dynint::DynInt, traits::{Deserialize, Serialize}};


impl Serialize for [u8] {
    fn serialize(&self, buf: &mut impl BufMut) {
        DynInt(self.len() as u64).serialize(buf);
        buf.put_slice(self);
    }
}

impl Serialize for Vec<u8> {
    fn serialize(&self, buf: &mut impl BufMut) {
        self[..].serialize(buf);
    }
}

impl Deserialize for Bytes {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
        let len = DynInt::deserialize(buf)?.0 as usize;
        
        if buf.remaining() < len {
            return Err(());
        }

        Ok(buf.slice(..len))
    }
}

impl Deserialize for Vec<u8> {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
        let len = DynInt::deserialize(buf)?.0 as usize;
        
        if buf.remaining() < len {
            return Err(());
        }
        
        let mut data = vec![0; len];
        buf.copy_to_slice(&mut data);
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use crate::traits::{Deserialize, Serialize};

    fn test_bytes(bytes: &[u8]) {
        let mut buf = BytesMut::with_capacity(bytes.len() + 9);

        bytes.serialize(&mut buf);
        assert_eq!(Bytes::deserialize(&mut buf.freeze()).unwrap(), bytes);
    }

    #[test]
    fn it_works() {
        test_bytes(&[]);
        test_bytes(&[0]);
        test_bytes(&[0, 1]);
        test_bytes(&[0; 10]);
        test_bytes(&[0; 10240]);
    }
}
