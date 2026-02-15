//! Base tool trait and types for TacoBot

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Result from tool execution
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Content for the LLM (what the tool learned/did)
    pub for_llm: String,
    /// Content to send to user immediately (optional)
    pub for_user: Option<String>,
    /// Whether this is an error
    pub is_error: bool,
    /// Whether to suppress user notification
    pub silent: bool,
    /// Whether execution is async
    pub async_exec: bool,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(for_llm: impl Into<String>) -> Self {
        Self {
            for_llm: for_llm.into(),
            for_user: None,
            is_error: false,
            silent: false,
            async_exec: false,
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            for_llm: message.into(),
            for_user: None,
            is_error: true,
            silent: false,
            async_exec: false,
        }
    }

    /// Add user-facing content
    pub fn with_user_content(mut self, content: impl Into<String>) -> Self {
        self.for_user = Some(content.into());
        self
    }

    /// Mark as silent (don't notify user)
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    /// Mark as async
    pub fn async_result(mut self) -> Self {
        self.async_exec = true;
        self
    }
}

/// Tool call from LLM response
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, Value>,
}

/// Tool trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (used by LLM to call it)
    fn name(&self) -> &str;

    /// Tool description (for LLM context)
    fn description(&self) -> &str;

    /// Tool parameters schema (JSON Schema format)
    fn parameters(&self) -> Value;

    /// Execute the tool
    async fn execute(&self, args: HashMap<String, Value>) -> ToolResult;
}

/// Optional trait for tools that need context
pub trait ContextualTool: Tool {
    fn set_context(&mut self, channel: &str, chat_id: &str);
}

/// Tool definition for LLM
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolDefinition {
    pub r#type: String,
    pub function: ToolFunctionDefinition,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl ToolDefinition {
    pub fn from_tool(tool: &dyn Tool) -> Self {
        Self {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            },
        }
    }
}
