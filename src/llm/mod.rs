//! LLM provider integrations

pub mod framework;
pub mod client;

pub use framework::LlmProvider;
pub use client::{LlmClient, LlmResponse};
