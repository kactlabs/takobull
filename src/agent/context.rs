//! Agent context management

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Message role enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
}

/// Context metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub channel: String,
    pub user_id: String,
    pub tags: Vec<String>,
}

/// Agent context for message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub session_id: String,
    pub user_input: String,
    pub conversation_history: Vec<Message>,
    pub available_tools: Vec<String>,
    pub metadata: ContextMetadata,
}
