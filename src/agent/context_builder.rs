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

## Response Style

- Be concise and direct
- Avoid emojis, actions (like *adjusts sunglasses*), or roleplay elements
- Focus on providing accurate information and executing tasks
- Keep responses professional and to the point

## Important Rules

1. **Use tools appropriately** - Only use tools when the user explicitly asks you to perform an action (write a file, execute a command, etc.). For questions, calculations, or information requests, respond directly without using tools.

2. **Examples of when NOT to use tools:**
   - "What is 2+2?" → Just answer "4"
   - "Explain how X works" → Just explain it
   - "What's the capital of France?" → Just answer "Paris"

3. **Examples of when TO use tools:**
   - "Write a file called test.txt" → Use write_file tool
   - "Execute this command" → Use appropriate tool
   - "Create a script that..." → Use write_file tool

4. **Tool Parameters** - When calling tools, provide the actual values for parameters, not the parameter descriptions."#,
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
        section.push_str("You have access to the following tools when you need to perform actions:\n\n");

        for tool_def in definitions {
            section.push_str(&format!(
                "- `{}` - {}\n",
                tool_def.function.name, tool_def.function.description
            ));
        }

        section
    }
}
