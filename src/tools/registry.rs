//! Tool registry for managing and executing tools

use super::base::{Tool, ToolDefinition, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Registry for managing tools
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool
    pub async fn register(&self, tool: Arc<dyn Tool>) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// Execute a tool
    pub async fn execute(&self, name: &str, args: HashMap<String, Value>) -> ToolResult {
        info!("Tool execution started: {}", name);

        let tool = match self.get(name).await {
            Some(t) => t,
            None => {
                error!("Tool not found: {}", name);
                return ToolResult::error(format!("Tool '{}' not found", name));
            }
        };

        let start = std::time::Instant::now();
        let result = tool.execute(args).await;
        let duration = start.elapsed();

        if result.is_error {
            error!(
                "Tool execution failed: {} ({}ms)",
                name,
                duration.as_millis()
            );
        } else {
            info!(
                "Tool execution completed: {} ({}ms, {} chars)",
                name,
                duration.as_millis(),
                result.for_llm.len()
            );
        }

        result
    }

    /// Get all tool definitions for LLM
    pub async fn get_definitions(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| ToolDefinition::from_tool(tool.as_ref()))
            .collect()
    }

    /// List all tool names
    pub async fn list(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Get tool count
    pub async fn count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
