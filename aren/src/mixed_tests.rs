use std::time;

use bytes::{Bytes, BytesMut};

use crate::{dynint::DynInt, traits::{Deserialize, Serialize}};

fn test_bytes(bytes: &[u8]) {
    let mut buf = BytesMut::with_capacity(bytes.len() + 9);

    bytes.serialize(&mut buf);
    assert_eq!(Bytes::deserialize(&mut buf.freeze()).unwrap(), bytes);
}

#[test]
fn it_works() {
    let mut buf = BytesMut::new();

    let now = time::Instant::now();
    DynInt(1351361).serialize(&mut buf);
    [0; 10240].serialize(&mut buf);
    "Hello world!".serialize(&mut buf);
    42_i8.serialize(&mut buf);
    42424_u16.serialize(&mut buf);
    42424242_i32.serialize(&mut buf);

    // println!("{buf:?}");
    println!("{:?}", now.elapsed());
}
