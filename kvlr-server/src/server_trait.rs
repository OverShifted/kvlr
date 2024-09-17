#![rustfmt::skip]

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures::future::FutureExt;

use kvlr::rpc::{
    connection_state::{into_handler, HandlerFn},
    pipelining::MaybePipelinedValue,
};

#[async_trait]
pub trait SomeFunctions: Send + Sync + 'static {
    
    async fn add(&self, arg0: u32, arg1: u32) -> u32;
    async fn append_string(&self, arg0: String, arg1: String) -> String;
    async fn range_vec(&self, arg0: u32) -> Vec<u32>;
    async fn call_me_to_panic(&self, ) -> ();

    #[allow(unused)]
    fn register(this: Arc<Self>, fns_map: &mut HashMap<u32, Arc<dyn HandlerFn>>) {
        
        {
            let this = this.clone();
            fns_map.insert(1337, into_handler(move |pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    let args: (u32, u32,) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<u32>, MaybePipelinedValue<u32>,) = rmp_serde::from_slice(&slice).unwrap();
                        (args.0.resolve(&pld).await.unwrap(),args.1.resolve(&pld).await.unwrap(),)
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = this.add(args.0,args.1,).shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
        {
            let this = this.clone();
            fns_map.insert(1234, into_handler(move |pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    let args: (String, String,) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<String>, MaybePipelinedValue<String>,) = rmp_serde::from_slice(&slice).unwrap();
                        (args.0.resolve(&pld).await.unwrap(),args.1.resolve(&pld).await.unwrap(),)
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = this.append_string(args.0,args.1,).shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
        {
            let this = this.clone();
            fns_map.insert(4321, into_handler(move |pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    let args: (u32,) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<u32>,) = rmp_serde::from_slice(&slice).unwrap();
                        (args.0.resolve(&pld).await.unwrap(),)
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = this.range_vec(args.0,).shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
        {
            let this = this.clone();
            fns_map.insert(1111, into_handler(move |pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    

                    let out = this.call_me_to_panic().shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
    }
}