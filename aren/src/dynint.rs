// 0 - 250 u8
// 251: u16
// 252: u32
// 253: u64

use bytes::{Buf, BufMut, Bytes};

use crate::traits::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynInt(pub u64);

impl Serialize for DynInt {
    fn serialize(&self, buf: &mut impl BufMut) {
        let number = self.0;

        match number {
            0..=250 => buf.put_u8(number as u8),
            251..65536 => {
                buf.put_u8(251);
                buf.put_u16_le(number as u16);
            },
            65536..4294967296 => {
                buf.put_u8(252);
                buf.put_u32_le(number as u32);
            },
            _ => {
                buf.put_u8(253);
                buf.put_u64_le(number);
            }
        }
    }
}

impl Deserialize for DynInt {
    fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
        if buf.remaining() == 0 {
            return Err(());
        }

        let head = buf.get_u8();
        Ok(DynInt(match head {
            0..=250 => head as u64,
            251 => {
                if buf.remaining() < 2 {
                    return Err(());
                }

                buf.get_u16_le() as u64
            },
            252 => {
                if buf.remaining() < 4 {
                    return Err(());
                }

                buf.get_u32_le() as u64
            },
            253 => {
                if buf.remaining() < 8 {
                    return Err(());
                }

                buf.get_u64_le()
            },
            _ => panic!()
        }))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::{dynint::DynInt, traits::{Deserialize, Serialize}};

    fn test_num(number: u64) {
        let number = DynInt(number);

        let mut buf = BytesMut::with_capacity(9);
        number.serialize(&mut buf);
        // println!("{number:?} -> {buf:?}");
        assert_eq!(DynInt::deserialize(&mut buf.freeze()), Ok(number));
    }

    #[test]
    fn it_works() {
        test_num(1);
        test_num(10);
        test_num(100);
        test_num(200);
        test_num(250);
        test_num(251);
        test_num(252);
        test_num(253);
        test_num(254);
        test_num(255);
        test_num(256);
        test_num(257);
        test_num(1000);
        test_num(65000);
        test_num(65534);
        test_num(65535);
        test_num(65536);
        test_num(65537);
        test_num(4294967294);
        test_num(4294967295);
        test_num(4294967296);
        test_num(4294967297);
        test_num(18446744073709551614);
        test_num(18446744073709551615);
    }
}
