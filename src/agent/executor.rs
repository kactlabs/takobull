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
        let mut conversation: Vec<serde_json::Value> = vec![
            json!({
                "role": "user",
                "content": message
            })
        ];

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

            // Call LLM with tools and conversation history
            let response = self
                .llm_client
                .chat_with_tools_and_history(message, tools_json, &conversation)
                .await?;

            // Build assistant message with tool calls if any
            let mut assistant_msg = json!({
                "role": "assistant",
                "content": response.content
            });

            if !response.tool_calls.is_empty() {
                let tool_calls_json: Vec<serde_json::Value> = response
                    .tool_calls
                    .iter()
                    .map(|tc| {
                        json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": serde_json::to_string(&tc.arguments).unwrap_or_default()
                            }
                        })
                    })
                    .collect();
                assistant_msg["tool_calls"] = json!(tool_calls_json);
            }

            conversation.push(assistant_msg);

            // If no tool calls, we're done
            if response.tool_calls.is_empty() {
                final_response = response.content;
                info!("LLM response without tool calls (iteration: {})", iteration);
                break;
            }

            // Log tool calls
            let tool_names: Vec<&str> = response.tool_calls.iter().map(|tc| tc.name.as_str()).collect();
            info!("LLM requested tool calls: {:?} (iteration: {})", tool_names, iteration);

            // Execute tools and collect results
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

                // Add tool result to conversation (OpenAI format)
                conversation.push(json!({
                    "role": "tool",
                    "tool_call_id": tool_call.id,
                    "content": result.for_llm
                }));
            }
        }

        Ok(final_response)
    }
}
