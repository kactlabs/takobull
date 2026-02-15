//! Channel manager for managing multiple channels

use crate::error::Result;
use crate::error::Error;
use super::framework::{Channel, OutgoingMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Channel manager for managing multiple channel implementations
pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<String, Arc<dyn Channel>>>>,
}

impl ChannelManager {
    /// Create a new channel manager
    pub fn new() -> Self {
        ChannelManager {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a channel
    pub async fn register(&self, channel_id: String, channel: Arc<dyn Channel>) -> Result<()> {
        let mut channels = self.channels.write().await;
        channels.insert(channel_id, channel);
        Ok(())
    }

    /// Get a channel by ID
    pub async fn get(&self, channel_id: &str) -> Result<Arc<dyn Channel>> {
        let channels = self.channels.read().await;
        channels
            .get(channel_id)
            .cloned()
            .ok_or_else(|| Error::channel(format!("Channel not found: {}", channel_id)))
    }

    /// List all channel IDs
    pub async fn list(&self) -> Result<Vec<String>> {
        let channels = self.channels.read().await;
        Ok(channels.keys().cloned().collect())
    }

    /// Send a message to a channel
    pub async fn send_message(&self, channel_id: &str, msg: OutgoingMessage) -> Result<()> {
        let channel = self.get(channel_id).await?;
        channel.send_message(msg).await
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}
