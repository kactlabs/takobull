//! TakoBull CLI entry point
//!
//! This is the main executable for TakoBull, providing command-line interface
//! and initialization of the system.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "takobull")]
#[command(about = "Ultra-lightweight personal AI Assistant for embedded systems", long_about = None)]
#[command(version)]
#[command(author)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<PathBuf>,

    /// Log level (debug, info, warn, error)
    #[arg(short, long, default_value = "info", global = true)]
    log_level: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Chat with the agent
    Agent {
        /// Message to send to the agent
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Start the gateway for channel integrations
    Gateway,
    /// Show system status
    Status,
    /// Manage scheduled cron jobs
    Cron {
        #[command(subcommand)]
        action: CronAction,
    },
    /// Initialize configuration and workspace
    Onboard,
}

#[derive(Subcommand, Debug)]
enum CronAction {
    /// List all scheduled jobs
    List,
    /// Add a new scheduled job
    Add {
        /// Cron expression
        #[arg(short, long)]
        expression: String,
        /// Job description
        #[arg(short, long)]
        description: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    picoclaw::logging::setup::init_logging(&args.log_level)?;

    info!("Starting TakoBull v{}", env!("CARGO_PKG_VERSION"));
    if let Some(config_path) = &args.config {
        info!("Configuration file: {:?}", config_path);
    }

    match args.command {
        Some(Commands::Agent { message }) => {
            handle_agent(message).await?;
        }
        Some(Commands::Gateway) => {
            handle_gateway().await?;
        }
        Some(Commands::Status) => {
            handle_status().await?;
        }
        Some(Commands::Cron { action }) => {
            handle_cron(action).await?;
        }
        Some(Commands::Onboard) => {
            handle_onboard().await?;
        }
        None => {
            // Default: show help
            println!("TakoBull v{}", env!("CARGO_PKG_VERSION"));
            println!("Ultra-lightweight personal AI Assistant for embedded systems");
            println!("\nUsage: takobull [OPTIONS] <COMMAND>");
            println!("\nCommands:");
            println!("  agent    Chat with the agent");
            println!("  gateway  Start the gateway for channel integrations");
            println!("  status   Show system status");
            println!("  cron     Manage scheduled cron jobs");
            println!("  onboard  Initialize configuration and workspace");
            println!("\nOptions:");
            println!("  -c, --config <FILE>          Path to configuration file");
            println!("  -l, --log-level <LOG_LEVEL>  Log level (debug, info, warn, error)");
            println!("  -v, --verbose                Enable verbose output");
            println!("  -h, --help                   Print help");
            println!("  -V, --version                Print version");
        }
    }

    info!("TakoBull completed successfully");

    Ok(())
}

async fn handle_agent(message: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting agent");

    // Load config
    let home = std::env::var("HOME")?;
    let config_path = format!("{}/.takobull/config.yaml", home);
    let workspace_path = format!("{}/.takobull/workspace", home);
    
    if !std::path::Path::new(&config_path).exists() {
        eprintln!("❌ Config not found: {}", config_path);
        eprintln!("Run 'takobull onboard' first to initialize");
        return Err("Config file not found".into());
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    info!("Loaded config from: {}", config_path);

    if let Some(msg) = message {
        info!("Processing message: {}", msg);
        
        // Parse YAML config
        let config: serde_yaml::Value = serde_yaml::from_str(&config_content)?;
        
        let provider = config["agents"]["defaults"]["provider"]
            .as_str()
            .unwrap_or("openrouter")
            .to_string();
        
        let model = config["agents"]["defaults"]["model"]
            .as_str()
            .unwrap_or("meta-llama/llama-2-70b-chat")
            .to_string();
        
        // Get API key and base from provider config
        let provider_config = &config["providers"][&provider];
        let api_key = provider_config["api_key"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let api_base = provider_config["api_base"]
            .as_str()
            .unwrap_or("https://openrouter.ai/api/v1")
            .to_string();
        
        info!("Using provider: {}, model: {}", provider, model);
        
        if api_key.is_empty() {
            eprintln!("❌ API key not configured for provider: {}", provider);
            eprintln!("Set the API key in ~/.takobull/config.yaml under providers.{}.api_key", provider);
            return Err("API key not configured".into());
        }
        
        // Create LLM client
        let llm_client = picoclaw::llm::LlmClient::new(&provider, &model, &api_key, &api_base);
        
        // Create tool registry and register tools
        let tool_registry = picoclaw::tools::ToolRegistry::new();
        let write_file_tool = std::sync::Arc::new(
            picoclaw::tools::WriteFileTool::new(workspace_path)
        );
        tool_registry.register(write_file_tool).await;
        
        // Create agent executor
        let executor = picoclaw::agent::AgentExecutor::new(llm_client, tool_registry);
        
        println!("🤖 Processing: {}", msg);
        
        match executor.execute(&msg).await {
            Ok(response) => {
                println!("{}", response);
                info!("Response: {}", response);
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("Starting interactive agent mode");
        println!("🤖 TakoBull Interactive Mode");
        println!("Type 'exit' to quit\n");
        
        // TODO: Start interactive REPL
        println!("(Interactive mode not yet implemented)");
    }

    Ok(())
}

async fn handle_gateway() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting gateway");
    println!("Gateway mode (not yet implemented)");
    // TODO: Initialize channel connections
    // TODO: Start listening for messages
    Ok(())
}

async fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    info!("Showing status");
    println!("TakoBull v{}", env!("CARGO_PKG_VERSION"));
    println!("Status: OK");
    // TODO: Show actual status information
    Ok(())
}

async fn handle_cron(action: CronAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CronAction::List => {
            info!("Listing cron jobs");
            println!("Cron jobs (not yet implemented)");
            // TODO: List scheduled jobs
        }
        CronAction::Add {
            expression,
            description,
        } => {
            info!("Adding cron job: {} - {}", expression, description);
            println!("Added cron job: {} - {}", expression, description);
            // TODO: Add scheduled job
        }
    }
    Ok(())
}

async fn handle_onboard() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting onboard process");
    
    let home = std::env::var("HOME")?;
    let workspace_dir = format!("{}/.takobull/workspace", home);
    let config_path = format!("{}/.takobull/config.yaml", home);
    
    // Create workspace directory
    std::fs::create_dir_all(&workspace_dir)?;
    println!("✓ Created workspace directory: {}", workspace_dir);
    
    // Create subdirectories
    let subdirs = vec!["sessions", "memory", "state", "cron", "skills"];
    for subdir in subdirs {
        std::fs::create_dir_all(format!("{}/{}", workspace_dir, subdir))?;
    }
    println!("✓ Created workspace subdirectories");
    
    // Create default config if it doesn't exist
    if !std::path::Path::new(&config_path).exists() {
        let default_config = r#"# TakoBull Configuration
# Ultra-lightweight personal AI Assistant for embedded systems

agents:
  defaults:
    workspace: "~/.takobull/workspace"
    restrict_to_workspace: true
    provider: "openrouter"
    model: "meta-llama/llama-2-70b-chat"
    max_tokens: 8192
    temperature: 0.7
    max_tool_iterations: 20

channels:
  telegram:
    enabled: false
    token: ""
    allow_from: []
  
  discord:
    enabled: false
    token: ""
    allow_from: []

providers:
  openrouter:
    api_key: ""
    api_base: "https://openrouter.ai/api/v1"
  
  anthropic:
    api_key: ""
    api_base: "https://api.anthropic.com"
  
  openai:
    api_key: ""
    api_base: "https://api.openai.com/v1"

tools:
  web:
    brave:
      enabled: true
      api_key: ""
      max_results: 5
    
    duckduckgo:
      enabled: true
      max_results: 5

heartbeat:
  enabled: true
  interval: 30

logging:
  level: "info"
  format: "json"
"#;
        std::fs::write(&config_path, default_config)?;
        println!("✓ Created default config: {}", config_path);
    } else {
        println!("✓ Config already exists: {}", config_path);
    }
    
    // Create workspace files
    let workspace_files = vec![
        ("AGENTS.md", "# Agent Configuration\n\nConfigure agent behavior here.\n"),
        ("IDENTITY.md", "# Agent Identity\n\nDefine your agent's identity and personality.\n"),
        ("SOUL.md", "# Agent Soul\n\nDefine your agent's core values and principles.\n"),
        ("TOOLS.md", "# Available Tools\n\nList of tools available to the agent.\n"),
        ("USER.md", "# User Preferences\n\nDefine user preferences and settings.\n"),
        ("HEARTBEAT.md", "# Periodic Tasks\n\nDefine tasks to run periodically.\n"),
        ("MEMORY.md", "# Long-term Memory\n\nAgent's long-term memory storage.\n"),
    ];
    
    for (filename, content) in workspace_files {
        let filepath = format!("{}/{}", workspace_dir, filename);
        if !std::path::Path::new(&filepath).exists() {
            std::fs::write(&filepath, content)?;
        }
    }
    println!("✓ Created workspace files");
    
    println!("\n✅ Onboarding complete!");
    println!("\nNext steps:");
    println!("1. Edit config: {}", config_path);
    println!("2. Set your API keys (OPENROUTER_API_KEY, etc.)");
    println!("3. Run: takobull agent -m \"Hello\"");
    
    info!("Onboarding completed successfully");
    Ok(())
}
