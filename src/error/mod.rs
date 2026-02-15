//! Error types and handling for TakoBull

use thiserror::Error;

pub mod types;

pub use types::PicoClawError;

/// Result type for TakoBull operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for TakoBull
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("Tool execution error: {0}")]
    Tool(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Error {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Create an authentication error
    pub fn auth(msg: impl Into<String>) -> Self {
        Error::Auth(msg.into())
    }

    /// Create a channel error
    pub fn channel(msg: impl Into<String>) -> Self {
        Error::Channel(msg.into())
    }

    /// Create an LLM provider error
    pub fn llm_provider(msg: impl Into<String>) -> Self {
        Error::LlmProvider(msg.into())
    }

    /// Create a tool execution error
    pub fn tool(msg: impl Into<String>) -> Self {
        Error::Tool(msg.into())
    }

    /// Create a session error
    pub fn session(msg: impl Into<String>) -> Self {
        Error::Session(msg.into())
    }

    /// Create a device error
    pub fn device(msg: impl Into<String>) -> Self {
        Error::Device(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Error::Serialization(msg.into())
    }

    /// Create an HTTP error
    pub fn http(msg: impl Into<String>) -> Self {
        Error::Http(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Error::Timeout(msg.into())
    }

    /// Create a runtime error
    pub fn runtime(msg: impl Into<String>) -> Self {
        Error::Runtime(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Error::Internal(msg.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err.to_string())
    }
}
