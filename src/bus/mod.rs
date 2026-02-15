//! Message bus for inter-component communication

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::error::Result;

/// Inbound message from a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub channel: String,
    pub chat_id: String,
    pub user_id: String,
    pub content: String,
}

/// Outbound message to a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub channel: String,
    pub chat_id: String,
    pub content: String,
}

/// Message bus for pub/sub communication
pub struct MessageBus {
    inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    inbound_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<InboundMessage>>>,
    outbound_tx: mpsc::UnboundedSender<OutboundMessage>,
    outbound_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<OutboundMessage>>>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel();

        MessageBus {
            inbound_tx,
            inbound_rx: Arc::new(tokio::sync::Mutex::new(inbound_rx)),
            outbound_tx,
            outbound_rx: Arc::new(tokio::sync::Mutex::new(outbound_rx)),
        }
    }

    /// Publish an inbound message
    pub fn publish_inbound(&self, msg: InboundMessage) -> Result<()> {
        self.inbound_tx
            .send(msg)
            .map_err(|_| crate::error::Error::internal("Failed to publish inbound message"))?;
        Ok(())
    }

    /// Consume an inbound message
    pub async fn consume_inbound(&self) -> Option<InboundMessage> {
        self.inbound_rx.lock().await.recv().await
    }

    /// Publish an outbound message
    pub fn publish_outbound(&self, msg: OutboundMessage) -> Result<()> {
        self.outbound_tx
            .send(msg)
            .map_err(|_| crate::error::Error::internal("Failed to publish outbound message"))?;
        Ok(())
    }

    /// Subscribe to outbound messages
    pub async fn subscribe_outbound(&self) -> Option<OutboundMessage> {
        self.outbound_rx.lock().await.recv().await
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}
