//! Heartbeat service for periodic checks

use crate::error::Result;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Heartbeat handler function type
pub type HeartbeatHandler = Arc<dyn Fn(String) -> Result<String> + Send + Sync>;

/// Heartbeat service for periodic checks
pub struct HeartbeatService {
    interval_minutes: u64,
    enabled: bool,
    handler: Arc<RwLock<Option<HeartbeatHandler>>>,
    running: Arc<RwLock<bool>>,
}

impl HeartbeatService {
    /// Create a new heartbeat service
    pub fn new(interval_minutes: u64, enabled: bool) -> Self {
        let interval = if interval_minutes < 5 && interval_minutes != 0 {
            5
        } else if interval_minutes == 0 {
            30
        } else {
            interval_minutes
        };

        HeartbeatService {
            interval_minutes: interval,
            enabled,
            handler: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Set the heartbeat handler
    pub async fn set_handler(&self, handler: HeartbeatHandler) {
        let mut h = self.handler.write().await;
        *h = Some(handler);
    }

    /// Start the heartbeat service
    pub async fn start(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        *running = true;

        let interval_minutes = self.interval_minutes;
        let handler = self.handler.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

            loop {
                ticker.tick().await;

                let is_running = *running.read().await;
                if !is_running {
                    break;
                }

                if let Some(h) = handler.read().await.as_ref() {
                    let _ = h("heartbeat".to_string());
                }
            }
        });

        Ok(())
    }

    /// Stop the heartbeat service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Check if service is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

impl Default for HeartbeatService {
    fn default() -> Self {
        Self::new(30, true)
    }
}
