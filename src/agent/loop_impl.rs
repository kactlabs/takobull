//! Agent loop implementation

use crate::error::Result;
use crate::config::Config;
use crate::session::SessionManager;
use crate::tools::ToolRegistry;
use crate::agent::context::{AgentContext, Message, MessageRole, ContextMetadata};
use crate::llm::framework::LlmProvider;
use std::sync::Arc;
use std::path::Path;

/// Agent loop for message processing
pub struct AgentLoop {
    config: Config,
    session_manager: Arc<SessionManager>,
    tool_registry: Arc<ToolRegistry>,
    llm_provider: Arc<dyn LlmProvider>,
    #[allow(dead_code)]
    workspace: String,
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
        })
    }

    /// Process a message
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
            timestamp: std::time::SystemTime::now(),
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

        // Call LLM
        let response = self
            .llm_provider
            .chat(
                &context.conversation_history,
                &self.tool_registry.get_definitions().await,
                &self.config.llm.default_provider,
            )
            .await?;

        // Add assistant response to session
        let assistant_msg = Message {
            role: MessageRole::Assistant,
            content: response.clone(),
            timestamp: std::time::SystemTime::now(),
        };
        session.messages.push(assistant_msg);

        // Save updated session
        self.session_manager.save_session(&session).await?;

        Ok(response)
    }

    /// Get session manager
    pub fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }

    /// Get tool registry
    pub fn tool_registry(&self) -> Arc<ToolRegistry> {
        self.tool_registry.clone()
    }
}

impl Default for AgentLoop {
    fn default() -> Self {
        panic!("AgentLoop requires configuration and LLM provider")
    }
}
