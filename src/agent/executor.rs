//! Agent executor with tool execution loop

use crate::llm::LlmClient;
use crate::tools::ToolRegistry;
use crate::agent::ContextBuilder;
use serde_json::json;
use tracing::{info, debug};
use std::sync::Arc;

pub struct AgentExecutor {
    llm_client: LlmClient,
    tool_registry: Arc<ToolRegistry>,
    context_builder: ContextBuilder,
    max_iterations: usize,
}

impl AgentExecutor {
    pub fn new(llm_client: LlmClient, tool_registry: Arc<ToolRegistry>) -> Self {
        let mut context_builder = ContextBuilder::new("/tmp");
        context_builder.set_tool_registry(tool_registry.clone());
        
        Self {
            llm_client,
            tool_registry,
            context_builder,
            max_iterations: 10,
        }
    }

    pub async fn execute(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Starting agent execution loop");

        let mut iteration = 0;
        let mut final_response = String::new();
        
        // Build system prompt
        let system_prompt = self.context_builder.build_system_prompt().await;
        
        // Check if provider supports native tool calling with tool role
        let supports_tool_role = matches!(
            self.llm_client.provider.as_str(),
            "openai" | "anthropic" | "openrouter" | "gemini" | "google"
        );
        
        // Check if provider supports tool calling (even if via prompt engineering)
        let enable_tools = matches!(
            self.llm_client.provider.as_str(),
            "openai" | "anthropic" | "openrouter" | "ollama" | "gemini" | "google" | "vllm"
        );
        
        let mut conversation: Vec<serde_json::Value> = vec![
            json!({
                "role": "system",
                "content": system_prompt
            }),
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

            let response = if enable_tools {
                // Get tool definitions
                let tool_defs = self.tool_registry.get_definitions().await;
                
                // Convert tool definitions to JSON for LLM
                let tools_json: Vec<serde_json::Value> = tool_defs
                    .iter()
                    .map(|td| serde_json::to_value(td).unwrap_or(json!({})))
                    .collect();

                // Call LLM with tools and conversation history
                self.llm_client
                    .chat_with_tools_and_history(message, tools_json, &conversation)
                    .await?
            } else {
                // For providers without tool support, use simple chat
                // Extract the last user message
                let last_message = conversation
                    .iter()
                    .rev()
                    .find(|msg| msg["role"] == "user")
                    .and_then(|msg| msg["content"].as_str())
                    .unwrap_or(message);
                
                let content = self.llm_client.chat(last_message).await?;
                
                crate::llm::LlmResponse {
                    content,
                    tool_calls: Vec::new(),
                }
            };

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
            
            // Debug: Log raw tool call arguments
            for tool_call in &response.tool_calls {
                info!("Tool call '{}' arguments: {}", tool_call.name, serde_json::to_string_pretty(&tool_call.arguments).unwrap_or_default());
            }

            // Execute tools and collect results
            let mut tool_results = Vec::new();
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

                if supports_tool_role {
                    // Add tool result to conversation (OpenAI format)
                    conversation.push(json!({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": result.for_llm
                    }));
                } else {
                    // Collect results for user message
                    tool_results.push(format!(
                        "Tool '{}' result: {}",
                        tool_call.name,
                        result.for_llm
                    ));
                }
            }
            
            // For providers without tool role support, add results as user message
            if !supports_tool_role && !tool_results.is_empty() {
                conversation.push(json!({
                    "role": "user",
                    "content": tool_results.join("\n\n")
                }));
            }
        }

        Ok(final_response)
    }
}
