use bytes::{Buf, BufMut, Bytes};

use crate::{dynint::DynInt, traits::{Deserialize, Serialize}};


impl Serialize for str {
    fn serialize(&self, buf: &mut impl BufMut) {
        self.as_bytes().serialize(buf);
    }
}

impl Deserialize for String {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
        let bytes = Vec::<u8>::deserialize(buf)?;
        String::from_utf8(bytes).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::traits::{Deserialize, Serialize};

    fn test_string(string: &str) {
        let mut buf = BytesMut::with_capacity(string.len() + 9);

        string.serialize(&mut buf);
        assert_eq!(String::deserialize(&mut buf.freeze()).unwrap(), string);
    }

    #[test]
    fn it_works() {
        test_string("");
        test_string("a");
        test_string("aa");
        test_string("aaa");
        test_string("qwertyuiopasdfghjklzxcvbnm");
    }
}
