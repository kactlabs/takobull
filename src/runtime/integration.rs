//! Integration module for bringing all components together

use crate::error::Result;
use crate::config::Config;
use crate::session::SessionManager;
use crate::state::StateManager;
use crate::agent::AgentLoop;
use crate::tools::ToolRegistry;
use crate::channels::ChannelManager;
use crate::cron::CronService;
use crate::heartbeat::HeartbeatService;
use crate::agent::MemoryManager;
use crate::bus::MessageBus;
use crate::llm::framework::LlmProvider;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Integrated runtime for PicoClaw
pub struct PicoClawRuntime {
    pub config: Config,
    pub workspace: PathBuf,
    pub session_manager: Arc<SessionManager>,
    pub state_manager: Arc<StateManager>,
    pub agent_loop: Arc<AgentLoop>,
    pub tool_registry: Arc<ToolRegistry>,
    pub channel_manager: Arc<ChannelManager>,
    pub cron_service: Arc<CronService>,
    pub heartbeat_service: Arc<HeartbeatService>,
    pub memory_manager: Arc<MemoryManager>,
    pub message_bus: Arc<MessageBus>,
}

impl PicoClawRuntime {
    /// Create a new PicoClaw runtime
    pub async fn new(
        config: Config,
        workspace: impl AsRef<Path>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Result<Self> {
        let workspace = workspace.as_ref().to_path_buf();

        // Initialize all components
        let session_manager = Arc::new(
            SessionManager::with_storage(workspace.join("sessions"))
                .unwrap_or_else(|_| SessionManager::new()),
        );

        let state_manager = Arc::new(StateManager::new(&workspace)?);

        let agent_loop = Arc::new(AgentLoop::new(config.clone(), &workspace, llm_provider).await?);

        let tool_registry = Arc::new(ToolRegistry::new());

        let channel_manager = Arc::new(ChannelManager::new());

        let cron_service = Arc::new(
            CronService::with_storage(workspace.join("cron"))
                .unwrap_or_else(|_| CronService::new()),
        );

        let heartbeat_service = Arc::new(HeartbeatService::new(
            config.agent.timeout_ms / 60000,
            true,
        ));

        let memory_manager = Arc::new(MemoryManager::new(
            &workspace,
            config.agent.memory_limit_mb,
        )?);

        let message_bus = Arc::new(MessageBus::new());

        Ok(PicoClawRuntime {
            config,
            workspace,
            session_manager,
            state_manager,
            agent_loop,
            tool_registry,
            channel_manager,
            cron_service,
            heartbeat_service,
            memory_manager,
            message_bus,
        })
    }

    /// Start the runtime
    pub async fn start(&self) -> Result<()> {
        // Start heartbeat service
        self.heartbeat_service.start().await?;

        Ok(())
    }

    /// Stop the runtime
    pub async fn stop(&self) -> Result<()> {
        // Stop heartbeat service
        self.heartbeat_service.stop().await?;

        Ok(())
    }

    /// Process a message through the agent loop
    pub async fn process_message(
        &self,
        session_id: &str,
        user_id: &str,
        channel: &str,
        message: &str,
    ) -> Result<String> {
        self.agent_loop
            .process_message(session_id, user_id, channel, message)
            .await
    }
}
