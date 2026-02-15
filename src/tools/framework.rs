//! Tool framework and abstractions

use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgs {
    pub params: HashMap<String, String>,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub metadata: HashMap<String, String>,
}

/// Tool trait for all tool implementations
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool
    async fn execute(&self, args: &ToolArgs) -> Result<ToolResult>;

    /// Get the tool name
    fn name(&self) -> &str;

    /// Get the tool description
    fn description(&self) -> &str;
}
