//! Agent executor with tool execution loop

use crate::llm::LlmClient;
use crate::tools::ToolRegistry;
use serde_json::json;
use tracing::{info, debug};

pub struct AgentExecutor {
    llm_client: LlmClient,
    tool_registry: ToolRegistry,
    max_iterations: usize,
}

impl AgentExecutor {
    pub fn new(llm_client: LlmClient, tool_registry: ToolRegistry) -> Self {
        Self {
            llm_client,
            tool_registry,
            max_iterations: 10,
        }
    }

    pub async fn execute(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Starting agent execution loop");

        let mut iteration = 0;
        let mut final_response = String::new();

        loop {
            iteration += 1;
            debug!("Agent iteration: {}", iteration);

            if iteration > self.max_iterations {
                info!("Max iterations reached");
                break;
            }

            // Get tool definitions
            let tool_defs = self.tool_registry.get_definitions().await;
            let tools_json: Vec<serde_json::Value> = tool_defs
                .iter()
                .map(|t| {
                    json!({
                        "type": t.r#type,
                        "function": {
                            "name": t.function.name,
                            "description": t.function.description,
                            "parameters": t.function.parameters,
                        }
                    })
                })
                .collect();

            // Call LLM with tools
            let response = self
                .llm_client
                .chat_with_tools(message, tools_json)
                .await?;

            // If no tool calls, we're done
            if response.tool_calls.is_empty() {
                final_response = response.content;
                info!("LLM response without tool calls (iteration: {})", iteration);
                break;
            }

            // Log tool calls
            let tool_names: Vec<&str> = response.tool_calls.iter().map(|tc| tc.name.as_str()).collect();
            info!("LLM requested tool calls: {:?} (iteration: {})", tool_names, iteration);

            // Execute tools
            for tool_call in &response.tool_calls {
                debug!("Executing tool: {}", tool_call.name);

                let result = self
                    .tool_registry
                    .execute(&tool_call.name, tool_call.arguments.clone())
                    .await;

                if result.is_error {
                    info!("Tool failed: {} - {}", tool_call.name, result.for_llm);
                } else {
                    info!("Tool succeeded: {}", tool_call.name);
                    if let Some(user_content) = &result.for_user {
                        println!("{}", user_content);
                    }
                }
            }
        }

        Ok(final_response)
    }
}
