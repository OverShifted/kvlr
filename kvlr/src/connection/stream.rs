use tokio::io::{AsyncRead, AsyncWrite};

// Combines AsyncRead and AsyncWrite
pub trait Stream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> Stream for T where T: AsyncRead + AsyncWrite + Send + Unpin {}
