//! Session manager implementation

use crate::error::Result;
use crate::error::Error;
use super::store::Session;
use crate::agent::context::Message;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session manager for managing conversation sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    storage_path: Option<PathBuf>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
        }
    }

    /// Create a new session manager with persistent storage
    pub fn with_storage(storage_path: impl AsRef<Path>) -> Result<Self> {
        let path = storage_path.as_ref();
        fs::create_dir_all(path)
            .map_err(|e| Error::session(format!("Failed to create storage directory: {}", e)))?;

        let mut manager = SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage_path: Some(path.to_path_buf()),
        };

        // Load existing sessions from storage
        manager.load_all_sessions()?;
        Ok(manager)
    }

    /// Create a new session
    pub async fn create_session(&self, user_id: &str) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now();

        let session = Session {
            id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            last_activity: now,
            messages: Vec::new(),
            metadata: super::store::SessionMetadata {
                channel: String::new(),
                tags: Vec::new(),
                custom_data: HashMap::new(),
            },
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        // Persist if storage is configured
        if let Some(path) = &self.storage_path {
            self.save_session_to_disk(path, &session)?;
        }

        Ok(session)
    }

    /// Load a session
    pub async fn load_session(&self, session_id: &str) -> Result<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| Error::session(format!("Session not found: {}", session_id)))
    }

    /// Get or create a session
    pub async fn get_or_create(&self, session_id: &str, user_id: &str) -> Result<Session> {
        if let Ok(session) = self.load_session(session_id).await {
            return Ok(session);
        }
        self.create_session(user_id).await
    }

    /// Save a session
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());

        if let Some(path) = &self.storage_path {
            self.save_session_to_disk(path, session)?;
        }

        Ok(())
    }

    /// Add a message to a session
    pub async fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::session(format!("Session not found: {}", session_id)))?;

        session.messages.push(message);
        session.last_activity = std::time::SystemTime::now();

        if let Some(path) = &self.storage_path {
            self.save_session_to_disk(path, session)?;
        }

        Ok(())
    }

    /// List all session IDs
    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.keys().cloned().collect())
    }

    // Private helper methods

    fn save_session_to_disk(&self, base_path: &Path, session: &Session) -> Result<()> {
        let session_file = base_path.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| Error::serialization(format!("Failed to serialize session: {}", e)))?;

        fs::write(&session_file, json)
            .map_err(|e| Error::session(format!("Failed to save session: {}", e)))?;

        Ok(())
    }

    fn load_all_sessions(&mut self) -> Result<()> {
        if let Some(path) = &self.storage_path {
            if !path.exists() {
                return Ok(());
            }

            for entry in fs::read_dir(path)
                .map_err(|e| Error::session(format!("Failed to read sessions directory: {}", e)))?
            {
                let entry = entry
                    .map_err(|e| Error::session(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();

                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(session) = serde_json::from_str::<Session>(&content) {
                            let mut sessions = futures::executor::block_on(self.sessions.write());
                            sessions.insert(session.id.clone(), session);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
