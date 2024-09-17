// use serde::{Deserialize, Serialize};

use std::{
    io::{Read, Write},
    string::FromUtf8Error,
    vec,
};

use bytes::Buf;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::connection::{StreamRead, StreamWrite};

#[derive(Debug)]
pub struct Frame {
    pub protocol: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum RecvFrameError {
    #[error("Error while reading frame: {0}")]
    IoError(std::io::Error),
    #[error("Error while converting frame's protocol to string: {0}")]
    InvalidProtocol(FromUtf8Error),
}

impl Frame {
    /// Read a frame by decoding it from a stream.
    pub async fn read_from_stream(stream: &mut dyn StreamRead) -> Result<Frame, RecvFrameError> {
        let frame_len = stream.read_u32().await.map_err(RecvFrameError::IoError)?;

        let mut frame_data = vec![0; frame_len as usize];

        stream
            .read_exact(&mut frame_data)
            .await
            .map_err(RecvFrameError::IoError)?;

        let mut reader = frame_data.reader();
        let mut reader = reader.get_mut();

        let protocol_len = reader.get_u32();
        let mut protocol = vec![0; protocol_len as usize];
        Read::read_exact(&mut reader, &mut protocol).map_err(RecvFrameError::IoError)?;

        let body_len = frame_len - protocol_len - 4;
        let mut body = vec![0; body_len as usize];
        Read::read_exact(&mut reader, &mut body).map_err(RecvFrameError::IoError)?;

        Ok(Frame {
            protocol: String::from_utf8(protocol).map_err(RecvFrameError::InvalidProtocol)?,
            body,
        })
    }

    /// Write a frame to a stream by encoding it.
    /// This method wont flush the stream.
    pub async fn write_to_stream(&self, stream: &mut dyn StreamWrite) -> std::io::Result<()> {
        let mut buffer = vec![0u8; 4 + self.len()];

        // TODO: (General) Client hangs on server proccess panic
        let mut writer = std::io::Cursor::new(&mut buffer);
        // Frame's length
        Write::write(&mut writer, &(self.len() as u32).to_be_bytes()).unwrap();
        // Frame's protocol's length
        Write::write(&mut writer, &(self.protocol.len() as u32).to_be_bytes()).unwrap();
        // Frame's protocol
        Write::write(&mut writer, self.protocol.as_bytes()).unwrap();
        // Frame's body
        Write::write(&mut writer, &self.body).unwrap();

        stream.write_all(&buffer).await?;
        Ok(())
    }

    /// Frame's size on network
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        4 + self.protocol.len() + self.body.len()
    }
}
