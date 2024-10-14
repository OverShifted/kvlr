use bytes::{Buf, BufMut, Bytes};
use crate::traits::{Deserialize, Serialize};

macro_rules! impl_int {
    ($type:ty, $putter:ident, $getter:ident) => {
        impl Serialize for $type {
            fn serialize(&self, buf: &mut impl BufMut) {
                buf.$putter(*self)
            }
        }

        impl Deserialize for $type {
            fn deserialize(buf: &mut Bytes) -> Result<Self, ()> {
                let len = std::mem::size_of::<$type>();
                
                if buf.remaining() < len {
                    return Err(());
                }
                
                Ok(buf.$getter())
            }
        }
    };
}

impl_int!(u8, put_u8, get_u8);
impl_int!(i8, put_i8, get_i8);

impl_int!(u16, put_u16_le, get_u16_le);
impl_int!(i16, put_i16_le, get_i16_le);

impl_int!(u32, put_u32_le, get_u32_le);
impl_int!(i32, put_i32_le, get_i32_le);

impl_int!(u64, put_u64_le, get_u64_le);
impl_int!(i64, put_i64_le, get_i64_le);
