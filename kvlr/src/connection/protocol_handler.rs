use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{frame::Frame, Connection};

#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    async fn handle_frame(&self, connection: &Arc<RwLock<Connection>>, frame: &Frame);
}
