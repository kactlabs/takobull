//! LLM provider framework and abstractions

use async_trait::async_trait;
use crate::error::Result;
use crate::agent::context::Message;
use serde::{Deserialize, Serialize};

/// LLM request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// LLM response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: TokenUsage,
}

/// LLM provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a response from the LLM
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Get the provider name
    fn provider_name(&self) -> &str;
}
