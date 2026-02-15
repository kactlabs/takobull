//! Channel framework and abstractions

use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Incoming message from a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub channel_id: String,
    pub user_id: String,
    pub content: String,
    pub timestamp: SystemTime,
}

/// Outgoing message to a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub channel_id: String,
    pub user_id: String,
    pub content: String,
}

/// Channel type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Telegram,
    Discord,
    DingTalk,
    Line,
    QQ,
    WhatsApp,
}

/// Channel trait for all channel implementations
#[async_trait]
pub trait Channel: Send + Sync {
    /// Connect to the channel
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the channel
    async fn disconnect(&mut self) -> Result<()>;

    /// Receive a message from the channel
    async fn receive_message(&mut self) -> Result<Option<IncomingMessage>>;

    /// Send a message to the channel
    async fn send_message(&self, msg: OutgoingMessage) -> Result<()>;

    /// Get the channel type
    fn channel_type(&self) -> ChannelType;
}
