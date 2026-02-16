//! Agent context management

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use crate::tools::ToolCall;

/// Message role enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
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
