use std::future::Future;

use async_trait::async_trait;

pub trait FutureSyncSend<Out>: Future<Output = Out> + Sync + Send {}
impl<T, Out> FutureSyncSend<Out> for T where T: Future<Output = Out> + Sync + Send {}

#[async_trait]
pub trait PromiseHelper<S, E> {
    fn on<SF, EF, SO, EO>(self, success: SF, fail: EF)
    where
        SF: FnOnce(S) -> SO + Send + 'static,
        EF: FnOnce(E) -> EO + Send + 'static,

        SO: FutureSyncSend<()> + 'static,
        EO: FutureSyncSend<()> + 'static;
}

#[async_trait]
impl<S, E, F> PromiseHelper<S, E> for F
where
    S: Send + 'static,
    E: Send + 'static,
    F: Future<Output = Result<S, E>> + Sized + Send + 'static {

    fn on<SF, EF, SO, EO>(self, success: SF, fail: EF)
    where
        SF: FnOnce(S) -> SO + Send + 'static,
        EF: FnOnce(E) -> EO + Send + 'static,

        SO: FutureSyncSend<()> + 'static,
        EO: FutureSyncSend<()> + 'static {

        tokio::spawn(async move {
            match self.await {
                Ok(s) => success(s).await,
                Err(e) => fail(e).await
            }
        });
    }
}
