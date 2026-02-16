//! Context builder for constructing system prompts and message sequences

use crate::tools::ToolRegistry;
use std::sync::Arc;

/// Context builder for constructing LLM prompts
pub struct ContextBuilder {
    workspace: String,
    tool_registry: Option<Arc<ToolRegistry>>,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new(workspace: impl Into<String>) -> Self {
        Self {
            workspace: workspace.into(),
            tool_registry: None,
        }
    }

    /// Set the tool registry for dynamic tool summaries
    pub fn set_tool_registry(&mut self, registry: Arc<ToolRegistry>) {
        self.tool_registry = Some(registry);
    }

    /// Build the system prompt
    pub async fn build_system_prompt(&self) -> String {
        let mut prompt = self.get_identity();

        // Add tools section
        if let Some(registry) = &self.tool_registry {
            let tools_section = self.build_tools_section(registry).await;
            if !tools_section.is_empty() {
                prompt.push_str("\n\n");
                prompt.push_str(&tools_section);
            }
        }

        prompt
    }

    /// Get the identity section
    fn get_identity(&self) -> String {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M (%A)");
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        format!(
            r#"# TakoBull 🐙

You are TakoBull, a helpful AI assistant.

## Current Time
{}

## Runtime
{} {}, Rust

## Workspace
Your workspace is at: {}

## Important Rules

1. **ALWAYS use tools** - When you need to perform an action (write files, execute commands, etc.), you MUST call the appropriate tool. Do NOT just say you'll do it or pretend to do it.

2. **Be helpful and accurate** - When using tools, briefly explain what you're doing.

3. **Tool Parameters** - When calling tools, provide the actual values for parameters, not the parameter descriptions."#,
            now, os, arch, self.workspace
        )
    }

    /// Build the tools section
    async fn build_tools_section(&self, registry: &ToolRegistry) -> String {
        let definitions = registry.get_definitions().await;
        if definitions.is_empty() {
            return String::new();
        }

        let mut section = String::from("## Available Tools\n\n");
        section.push_str("**CRITICAL**: You MUST use tools to perform actions. Do NOT pretend to execute commands or write files.\n\n");
        section.push_str("You have access to the following tools:\n\n");

        for tool_def in definitions {
            section.push_str(&format!(
                "- `{}` - {}\n",
                tool_def.function.name, tool_def.function.description
            ));
        }

        section
    }
}
