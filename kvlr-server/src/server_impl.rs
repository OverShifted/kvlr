use async_trait::async_trait;

use crate::server_trait::SomeFunctions;

pub struct ServerImpl;

#[async_trait]
impl SomeFunctions for ServerImpl {
    async fn add(&self, arg0: u32, arg1: u32) -> u32 {
        arg0 + arg1
    }

    async fn append_string(&self, arg0: String, arg1: String) -> String {
        arg0 + &arg1
    }

    async fn range_vec(&self, arg0: u32) -> Vec<u32> {
        (0..arg0).collect()
    }

    async fn call_me_to_panic(&self) -> () {
        panic!("I was intended to panic!")
    }
}
