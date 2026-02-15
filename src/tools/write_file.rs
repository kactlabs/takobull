//! Write file tool for TacoBot

use super::base::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

/// Write file tool
pub struct WriteFileTool {
    workspace: String,
}

impl WriteFileTool {
    pub fn new(workspace: String) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file in the workspace"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to workspace"
                },
                "content": {
                    "type": "string",
                    "description": "File content to write"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: HashMap<String, Value>) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing 'path' parameter"),
        };

        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::error("Missing 'content' parameter"),
        };

        // Validate path is within workspace
        let full_path = std::path::PathBuf::from(&self.workspace).join(path);
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return ToolResult::error(format!("Failed to create directories: {}", e));
                }
            }
        }

        // Check if path is within workspace
        let workspace_path = std::path::Path::new(&self.workspace);
        let full_path_str = full_path.to_string_lossy();
        let workspace_str = workspace_path.to_string_lossy();
        
        if !full_path_str.starts_with(workspace_str.as_ref()) {
            return ToolResult::error("Path is outside workspace");
        }

        // Write file
        match std::fs::write(&full_path, content) {
            Ok(_) => {
                info!("File written: {}", path);
                ToolResult::success(format!("File written successfully: {}", path))
                    .with_user_content(format!("✓ Created file: {}", path))
            }
            Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
        }
    }
}
