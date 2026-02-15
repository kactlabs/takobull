//! Configuration management for TacoBot

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        let result: Result<Config, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_yaml_config() {
        let invalid_yaml = r#"
agent:
  max_context_size: not_a_number
"#;
        let result: Result<Config, _> = serde_yaml::from_str(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_toml_config() {
        let invalid_toml = r#"
[agent]
max_context_size = "not_a_number"
"#;
        let result: Result<Config, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields_json() {
        let incomplete_json = r#"{ "agent": {} }"#;
        let result: Result<Config, _> = serde_json::from_str(incomplete_json);
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
