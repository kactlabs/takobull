//! Configuration management for TacoBot

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use crate::error::{Error, Result};

#[cfg(test)]
mod property_tests;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub channels: ChannelsConfig,
    pub llm: LlmConfig,
    pub tools: ToolsConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_context_size: usize,
    pub timeout_ms: u64,
    pub memory_limit_mb: usize,
}

/// Channels configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelsConfig {
    pub telegram: Option<ChannelConfig>,
    pub discord: Option<ChannelConfig>,
}

/// Individual channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub enabled: bool,
    pub token: Option<String>,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model: Option<String>,
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub web_search: Option<ToolConfig>,
    pub filesystem: Option<ToolConfig>,
    pub shell: Option<ToolConfig>,
}

/// Individual tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub enabled: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub oauth_enabled: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    /// Load configuration from file (YAML, JSON, or TOML)
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| Error::config(format!("Failed to read config file: {}", e)))?;

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("yaml");

        match ext {
            "json" => serde_json::from_str(&content)
                .map_err(|e| Error::config(format!("Invalid JSON config: {}", e))),
            "yaml" | "yml" => serde_yaml::from_str(&content)
                .map_err(|e| Error::config(format!("Invalid YAML config: {}", e))),
            "toml" => toml::from_str(&content)
                .map_err(|e| Error::config(format!("Invalid TOML config: {}", e))),
            _ => Err(Error::config(format!(
                "Unsupported config format: {}",
                ext
            ))),
        }
    }

    /// Load configuration from workspace directory
    pub fn from_workspace(workspace: impl AsRef<Path>) -> Result<Self> {
        let workspace = workspace.as_ref();
        let config_paths = [
            workspace.join("config.yaml"),
            workspace.join("config.yml"),
            workspace.join("config.json"),
            workspace.join("config.toml"),
        ];

        for path in &config_paths {
            if path.exists() {
                return Self::from_file(path);
            }
        }

        Ok(Self::default())
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(mut self) -> Self {
        if let Ok(val) = env::var("PICOCLAW_AGENT_MAX_CONTEXT_SIZE") {
            if let Ok(size) = val.parse() {
                self.agent.max_context_size = size;
            }
        }
        if let Ok(val) = env::var("PICOCLAW_AGENT_TIMEOUT_MS") {
            if let Ok(timeout) = val.parse() {
                self.agent.timeout_ms = timeout;
            }
        }
        if let Ok(val) = env::var("PICOCLAW_LLM_DEFAULT_PROVIDER") {
            self.llm.default_provider = val;
        }
        if let Ok(val) = env::var("PICOCLAW_LOGGING_LEVEL") {
            self.logging.level = val;
        }
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            agent: AgentConfig {
                max_context_size: 8192,
                timeout_ms: 5000,
                memory_limit_mb: 10,
            },
            channels: ChannelsConfig {
                telegram: None,
                discord: None,
            },
            llm: LlmConfig {
                default_provider: "openrouter".to_string(),
                providers: HashMap::new(),
            },
            tools: ToolsConfig {
                web_search: None,
                filesystem: None,
                shell: None,
            },
            auth: AuthConfig {
                oauth_enabled: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_json_config() {
        let invalid_json = r#"{ invalid json }"#;
        let result: std::result::Result<Config, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_yaml_config() {
        let invalid_yaml = r#"
agent:
  max_context_size: not_a_number
"#;
        let result: std::result::Result<Config, _> = serde_yaml::from_str(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_toml_config() {
        let invalid_toml = r#"
[agent]
max_context_size = "not_a_number"
"#;
        let result: std::result::Result<Config, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields_json() {
        let incomplete_json = r#"{ "agent": {} }"#;
        let result: std::result::Result<Config, _> = serde_json::from_str(incomplete_json);
        // This should fail because required fields are missing
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config_is_valid() {
        let config = Config::default();
        assert!(config.agent.max_context_size > 0);
        assert!(config.agent.timeout_ms > 0);
        assert!(config.agent.memory_limit_mb > 0);
        assert!(!config.llm.default_provider.is_empty());
    }

    #[test]
    fn test_config_serialization_preserves_values() {
        let mut config = Config::default();
        config.agent.max_context_size = 16384;
        config.agent.timeout_ms = 10000;
        config.agent.memory_limit_mb = 20;

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent.max_context_size, 16384);
        assert_eq!(deserialized.agent.timeout_ms, 10000);
        assert_eq!(deserialized.agent.memory_limit_mb, 20);
    }
}
