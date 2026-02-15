//! Session storage and persistence

use crate::agent::context::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub channel: String,
    pub tags: Vec<String>,
    pub custom_data: HashMap<String, String>,
}

/// Session structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub messages: Vec<Message>,
    pub metadata: SessionMetadata,
}
