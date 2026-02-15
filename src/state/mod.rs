//! State management for TakoBull

use crate::error::Result;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::SystemTime;

/// State structure for tracking workspace state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub last_channel: Option<String>,
    pub last_chat_id: Option<String>,
    pub timestamp: SystemTime,
}

impl Default for State {
    fn default() -> Self {
        State {
            last_channel: None,
            last_chat_id: None,
            timestamp: SystemTime::now(),
        }
    }
}

/// State manager for persistent state
pub struct StateManager {
    state: Arc<RwLock<State>>,
    state_file: PathBuf,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(workspace: impl AsRef<Path>) -> Result<Self> {
        let workspace = workspace.as_ref();
        let state_dir = workspace.join("state");
        fs::create_dir_all(&state_dir)
            .map_err(|e| Error::internal(format!("Failed to create state directory: {}", e)))?;

        let state_file = state_dir.join("state.json");
        let mut state = State::default();

        // Try to load existing state
        if state_file.exists() {
            if let Ok(content) = fs::read_to_string(&state_file) {
                if let Ok(loaded_state) = serde_json::from_str::<State>(&content) {
                    state = loaded_state;
                }
            }
        }

        Ok(StateManager {
            state: Arc::new(RwLock::new(state)),
            state_file,
        })
    }

    /// Get the last channel
    pub async fn get_last_channel(&self) -> Option<String> {
        self.state.read().await.last_channel.clone()
    }

    /// Set the last channel
    pub async fn set_last_channel(&self, channel: String) -> Result<()> {
        let mut state = self.state.write().await;
        state.last_channel = Some(channel);
        state.timestamp = SystemTime::now();
        self.save_state(&state).await
    }

    /// Get the last chat ID
    pub async fn get_last_chat_id(&self) -> Option<String> {
        self.state.read().await.last_chat_id.clone()
    }

    /// Set the last chat ID
    pub async fn set_last_chat_id(&self, chat_id: String) -> Result<()> {
        let mut state = self.state.write().await;
        state.last_chat_id = Some(chat_id);
        state.timestamp = SystemTime::now();
        self.save_state(&state).await
    }

    /// Get current state
    pub async fn get_state(&self) -> State {
        self.state.read().await.clone()
    }

    // Private helper
    async fn save_state(&self, state: &State) -> Result<()> {
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| Error::serialization(format!("Failed to serialize state: {}", e)))?;

        fs::write(&self.state_file, json)
            .map_err(|e| Error::internal(format!("Failed to save state: {}", e)))?;

        Ok(())
    }
}
