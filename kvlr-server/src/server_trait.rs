use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures::future::FutureExt;

use kvlr::{connection::Connection, rpc::{
    connection_state::{into_handler, HandlerFn},
    pipelining::MaybePipelinedValue,
}};

#[async_trait]
pub trait SomeFunctions: Send + Sync + 'static {
    
    async fn add(&self, connection: Arc<Connection>, arg0: u32, arg1: u32) -> u32;
    async fn append_string(&self, connection: Arc<Connection>, arg0: String, arg1: String) -> String;
    async fn login(&self, connection: Arc<Connection>, arg0: String, arg1: String) -> bool;

    #[allow(unused)]
    fn register(this: Arc<Self>, fns_map: &mut HashMap<u32, Arc<dyn HandlerFn>>) {
        
        {
            let this = this.clone();
            fns_map.insert(1337, into_handler(move |conn, pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    let args: (u32, u32,) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<u32>, MaybePipelinedValue<u32>,) = rmp_serde::from_slice(&slice).unwrap();
                        (args.0.resolve(&pld).await.unwrap(),args.1.resolve(&pld).await.unwrap(),)
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = this.add(conn, args.0,args.1,).shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
        {
            let this = this.clone();
            fns_map.insert(1234, into_handler(move |conn, pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    let args: (String, String,) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<String>, MaybePipelinedValue<String>,) = rmp_serde::from_slice(&slice).unwrap();
                        (args.0.resolve(&pld).await.unwrap(),args.1.resolve(&pld).await.unwrap(),)
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = this.append_string(conn, args.0,args.1,).shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
        {
            let this = this.clone();
            fns_map.insert(2000, into_handler(move |conn, pld, slice: Vec<u8>| {
                let this = this.clone();
                async move {
                    let args: (String, String,) = if let Some(pld) = pld {
                        let args: (MaybePipelinedValue<String>, MaybePipelinedValue<String>,) = rmp_serde::from_slice(&slice).unwrap();
                        (args.0.resolve(&pld).await.unwrap(),args.1.resolve(&pld).await.unwrap(),)
                    } else {
                        rmp_serde::from_slice(&slice).unwrap()
                    };

                    let out = this.login(conn, args.0,args.1,).shared().await;
                    rmp_serde::to_vec(&out).unwrap()
                }
            }));
        }
        
    }
}