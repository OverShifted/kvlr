use std::sync::Arc;

use tokio::{
    io::AsyncWriteExt,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot, RwLock,
    },
};
use tracing::error;

use super::{frame::Frame, protocol_handler::ProtocolHandler};

use crate::{
    connection::{Connection, StreamRead, StreamWrite}, rpc::protocol_handler::RpcProtocolHandler,
};

// TODO: think (more) about termination
pub(super) async fn read_processor(
    this: Arc<RwLock<Connection>>,
    mut stream: Box<dyn StreamRead>,
    _tx: Sender<Frame>,
) {
    loop {
        let frame = match Frame::read_from_stream(&mut stream).await {
            Ok(frame) => frame,
            Err(error) => {
                error!("Error reading frame: {}", error);
                break;
            }
        };

        match frame.protocol.as_str() {
            "close" => break,
            "rpc" => RpcProtocolHandler.handle_frame(&this, &frame).await,
            _ => (),
        }
    }
}

async fn write_and_flush(this: &mut Box<dyn StreamWrite>, frame: &Frame) -> std::io::Result<()> {
    frame.write_to_stream(this).await?;
    this.flush().await?;

    Ok(())
}

pub(super) async fn write_processor(
    mut this: Box<dyn StreamWrite>,
    mut rx: Receiver<(Frame, oneshot::Sender<std::io::Result<()>>)>,
) {
    loop {
        match rx.recv().await {
            Some((frame, tx)) => {
                // frame.write_to_stream(&mut this).await.unwrap();
                // this.flush().await.unwrap();

                // let _ = tx.send(Ok(()));

                // FIXME: It seems like calling a new async fn has a noticable amount of overhead
                let _ = tx.send(write_and_flush(&mut this, &frame).await);
            }

            None => break,
        };
    }
}
