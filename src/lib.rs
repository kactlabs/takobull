//! TakoBull - Ultra-lightweight personal AI Assistant for embedded systems
//!
//! This library provides the core functionality for TacoBot, including:
//! - Async runtime management
//! - Configuration loading and validation
//! - Authentication (OAuth2 with PKCE)
//! - Agent loop for message processing
//! - Channel integrations (Telegram, Discord, etc.)
//! - LLM provider integrations
//! - Tool framework for extensibility
//! - Session and state management
//! - Device management for hardware interfaces

pub mod agent;
pub mod auth;
pub mod channels;
pub mod config;
pub mod device;
pub mod error;
pub mod llm;
pub mod logging;
pub mod runtime;
pub mod session;
pub mod tools;

pub use error::{Error, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{Error, Result};
}
