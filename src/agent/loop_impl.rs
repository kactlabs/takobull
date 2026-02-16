//! Agent loop implementation with full tool calling support

use crate::error::Result;
use crate::config::Config;
use crate::session::SessionManager;
use crate::tools::ToolRegistry;
use crate::agent::context::{AgentContext, Message, MessageRole, ContextMetadata};
use crate::llm::framework::LlmProvider;
use std::sync::Arc;
use std::path::Path;
use std::time::SystemTime;
use tracing::{info, debug, error};

/// Agent loop for message processing with tool calling support
pub struct AgentLoop {
    config: Config,
    session_manager: Arc<SessionManager>,
    tool_registry: Arc<ToolRegistry>,
    llm_provider: Arc<dyn LlmProvider>,
    #[allow(dead_code)]
    workspace: String,
    max_iterations: usize,
}

impl AgentLoop {
    /// Create a new agent loop
    pub async fn new(
        config: Config,
        workspace: impl AsRef<Path>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Result<Self> {
        let workspace_str = workspace.as_ref().to_string_lossy().to_string();
        let sessions_path = Path::new(&workspace_str).join("sessions");

        let session_manager = Arc::new(
            SessionManager::with_storage(&sessions_path)
                .unwrap_or_else(|_| SessionManager::new()),
        );

        let tool_registry = Arc::new(ToolRegistry::new());

        Ok(AgentLoop {
            config,
            session_manager,
            tool_registry,
            llm_provider,
            workspace: workspace_str,
            max_iterations: 10, // Prevent infinite loops
        })
    }

    /// Process a message with full tool calling loop
    pub async fn process_message(
        &self,
        session_id: &str,
        user_id: &str,
        channel: &str,
        user_message: &str,
    ) -> Result<String> {
        // Get or create session
        let mut session = self
            .session_manager
            .get_or_create(session_id, user_id)
            .await?;

        // Update session metadata
        session.metadata.channel = channel.to_string();

        // Add user message to session
        let user_msg = Message {
            role: MessageRole::User,
            content: user_message.to_string(),
            timestamp: SystemTime::now(),
            tool_calls: None,
            tool_call_id: None,
        };
        session.messages.push(user_msg.clone());

        // Save session
        self.session_manager.save_session(&session).await?;

        // Build context
        let context = AgentContext {
            session_id: session_id.to_string(),
            user_input: user_message.to_string(),
            conversation_history: session.messages.clone(),
            available_tools: self.tool_registry.list().await,
            metadata: ContextMetadata {
                channel: channel.to_string(),
                user_id: user_id.to_string(),
                tags: session.metadata.tags.clone(),
            },
        };

        // Run the LLM iteration loop
        let final_response = self
            .run_llm_iteration_loop(
                context.conversation_history,
                &self.config.llm.default_provider,
            )
            .await?;

        // Add assistant response to session
        let assistant_msg = Message {
            role: MessageRole::Assistant,
            content: final_response.clone(),
            timestamp: SystemTime::now(),
            tool_calls: None,
            tool_call_id: None,
        };
        session.messages.push(assistant_msg);

        // Save updated session
        self.session_manager.save_session(&session).await?;

        Ok(final_response)
    }

    /// Run the LLM iteration loop with tool calling
    async fn run_llm_iteration_loop(
        &self,
        mut messages: Vec<Message>,
        model: &str,
    ) -> Result<String> {
        let mut iteration = 0;
        let mut final_content = String::new();

        loop {
            iteration += 1;
            debug!("LLM iteration: {}/{}", iteration, self.max_iterations);

            if iteration > self.max_iterations {
                error!("Max iterations reached");
                break;
            }

            // Get tool definitions
            let tool_defs = self.tool_registry.get_definitions().await;
            debug!("Available tools: {}", tool_defs.len());

            // Call LLM
            let response = self
                .llm_provider
                .chat(&messages, &tool_defs, model)
                .await?;

            final_content = response.content.clone();

            // Check if there are tool calls
            if response.tool_calls.is_empty() {
                info!("LLM response without tool calls (iteration: {})", iteration);
                break;
            }

            // Log tool calls
            let tool_names: Vec<_> = response.tool_calls.iter().map(|tc| tc.name.clone()).collect();
            info!("LLM requested tool calls: {:?} (iteration: {})", tool_names, iteration);

            // Build assistant message with tool calls
            let assistant_msg = Message {
                role: MessageRole::Assistant,
                content: response.content.clone(),
                timestamp: SystemTime::now(),
                tool_calls: Some(response.tool_calls.clone()),
                tool_call_id: None,
            };
            messages.push(assistant_msg);

            // Execute each tool call
            for tool_call in response.tool_calls {
                debug!("Executing tool: {}", tool_call.name);

                let tool_result = self
                    .tool_registry
                    .execute(&tool_call.name, tool_call.arguments)
                    .await;

                // Build tool result message
                let tool_result_msg = Message {
                    role: MessageRole::Tool,
                    content: if tool_result.is_error {
                        tool_result.for_llm.clone()
                    } else {
                        tool_result.for_llm.clone()
                    },
                    timestamp: SystemTime::now(),
                    tool_calls: None,
                    tool_call_id: Some(tool_call.id.clone()),
                };
                messages.push(tool_result_msg);

                // Log tool result
                if tool_result.is_error {
                    error!("Tool {} failed: {}", tool_call.name, tool_result.for_llm);
                } else {
                    info!(
                        "Tool {} completed: {} chars",
                        tool_call.name,
                        tool_result.for_llm.len()
                    );
                }
            }
        }

        Ok(final_content)
    }

    /// Get session manager
    pub fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }

    /// Get tool registry
    pub fn tool_registry(&self) -> Arc<ToolRegistry> {
        self.tool_registry.clone()
    }

    /// Set max iterations for the loop
    pub fn set_max_iterations(&mut self, max: usize) {
        self.max_iterations = max;
    }
}

impl Default for AgentLoop {
    fn default() -> Self {
        panic!("AgentLoop requires configuration and LLM provider")
    }
}
