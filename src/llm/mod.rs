//! LLM provider integrations

pub mod framework;
pub mod client;

#[cfg(test)]
mod ollama_test;

pub use framework::LlmProvider;
pub use client::{LlmClient, LlmResponse};
