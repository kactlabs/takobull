//! Simple LLM client for making requests to various providers

use serde_json::json;
use crate::error::{Error, Result};
use crate::tools::ToolCall;
use std::collections::HashMap;

pub struct LlmResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

pub struct LlmClient {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub api_base: String,
}

impl LlmClient {
    pub fn new(provider: &str, model: &str, api_key: &str, api_base: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
            api_base: api_base.to_string(),
        }
    }

    /// Strip provider prefix from model name (e.g., vllm/qwen-7b -> qwen-7b)
    pub fn normalize_model_name(&self, model: &str) -> String {
        if let Some(idx) = model.find('/') {
            let prefix = &model[..idx];
            // Strip prefix for providers that use it as a namespace
            if prefix == "vllm" || prefix == "moonshot" || prefix == "nvidia" || prefix == "ollama" {
                model[idx + 1..].to_string()
            } else {
                model.to_string()
            }
        } else {
            model.to_string()
        }
    }

    /// Get the correct chat completions endpoint for the provider
    pub fn get_chat_endpoint(&self) -> String {
        let base = self.api_base.trim_end_matches('/');
        match self.provider.as_str() {
            "vllm" | "ollama" => format!("{}/v1/chat/completions", base),
            _ => format!("{}/chat/completions", base),
        }
    }

    pub async fn chat(&self, message: &str) -> Result<String> {
        match self.provider.as_str() {
            "openrouter" => self.chat_openrouter(message).await,
            "openai" => self.chat_openai(message).await,
            "anthropic" => self.chat_anthropic(message).await,
            "vllm" => self.chat_openai(message).await, // vllm uses OpenAI-compatible API
            "ollama" => self.chat_ollama(message).await,
            _ => Err(Error::llm_provider(format!(
                "Unsupported provider: {}",
                self.provider
            ))),
        }
    }

    pub async fn chat_with_tools(
        &self,
        message: &str,
        tools: Vec<serde_json::Value>,
    ) -> Result<LlmResponse> {
        match self.provider.as_str() {
            "openrouter" => self.chat_openrouter_with_tools(message, tools).await,
            "openai" => self.chat_openai_with_tools(message, tools).await,
            "anthropic" => self.chat_anthropic_with_tools(message, tools).await,
            "vllm" => self.chat_openai_with_tools(message, tools).await, // vllm uses OpenAI-compatible API
            "ollama" => self.chat_ollama_with_tools(message, tools).await,
            _ => Err(Error::llm_provider(format!(
                "Unsupported provider: {}",
                self.provider
            ))),
        }
    }

    pub async fn chat_with_tools_and_history(
        &self,
        _message: &str,
        tools: Vec<serde_json::Value>,
        history: &[serde_json::Value],
    ) -> Result<LlmResponse> {
        match self.provider.as_str() {
            "openrouter" => self.chat_openrouter_with_tools_and_history(tools, history).await,
            "openai" => self.chat_openai_with_tools_and_history(tools, history).await,
            "anthropic" => self.chat_anthropic_with_tools_and_history(tools, history).await,
            "vllm" => self.chat_openai_with_tools_and_history(tools, history).await, // vllm uses OpenAI-compatible API
            "ollama" => self.chat_ollama_with_tools_and_history(tools, history).await,
            _ => Err(Error::llm_provider(format!(
                "Unsupported provider: {}",
                self.provider
            ))),
        }
    }

    async fn chat_openrouter_with_tools_and_history(
        &self,
        tools: Vec<serde_json::Value>,
        history: &[serde_json::Value],
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": history,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut tool_calls = Vec::new();
        if let Some(calls) = data["choices"][0]["message"]["tool_calls"].as_array() {
            for call in calls {
                if let (Some(id), Some(name), Some(args)) = (
                    call["id"].as_str(),
                    call["function"]["name"].as_str(),
                    call["function"]["arguments"].as_str(),
                ) {
                    let arguments: HashMap<String, serde_json::Value> =
                        serde_json::from_str(args).unwrap_or_default();
                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        name: name.to_string(),
                        arguments,
                    });
                }
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    async fn chat_openai_with_tools_and_history(
        &self,
        tools: Vec<serde_json::Value>,
        history: &[serde_json::Value],
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": history,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut tool_calls = Vec::new();
        if let Some(calls) = data["choices"][0]["message"]["tool_calls"].as_array() {
            for call in calls {
                if let (Some(id), Some(name), Some(args)) = (
                    call["id"].as_str(),
                    call["function"]["name"].as_str(),
                    call["function"]["arguments"].as_str(),
                ) {
                    let arguments: HashMap<String, serde_json::Value> =
                        serde_json::from_str(args).unwrap_or_default();
                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        name: name.to_string(),
                        arguments,
                    });
                }
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    async fn chat_anthropic_with_tools_and_history(
        &self,
        tools: Vec<serde_json::Value>,
        history: &[serde_json::Value],
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/messages", self.api_base);
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "max_tokens": 2048,
            "tools": tools,
            "messages": history,
        });

        let response = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(blocks) = data["content"].as_array() {
            for block in blocks {
                if let Some(text) = block["text"].as_str() {
                    content.push_str(text);
                }
                if block["type"].as_str() == Some("tool_use") {
                    if let (Some(id), Some(name), Some(input)) = (
                        block["id"].as_str(),
                        block["name"].as_str(),
                        block["input"].as_object(),
                    ) {
                        let mut arguments = HashMap::new();
                        for (k, v) in input {
                            arguments.insert(k.clone(), v.clone());
                        }
                        tool_calls.push(ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments,
                        });
                    }
                }
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    async fn chat_openrouter(&self, message: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::llm_provider("No content in response".to_string()))
    }

    async fn chat_openrouter_with_tools(
        &self,
        message: &str,
        tools: Vec<serde_json::Value>,
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut tool_calls = Vec::new();
        if let Some(calls) = data["choices"][0]["message"]["tool_calls"].as_array() {
            for call in calls {
                if let (Some(id), Some(name), Some(args)) = (
                    call["id"].as_str(),
                    call["function"]["name"].as_str(),
                    call["function"]["arguments"].as_str(),
                ) {
                    let arguments: HashMap<String, serde_json::Value> =
                        serde_json::from_str(args).unwrap_or_default();
                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        name: name.to_string(),
                        arguments,
                    });
                }
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    async fn chat_openai(&self, message: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::llm_provider("No content in response".to_string()))
    }

    async fn chat_openai_with_tools(
        &self,
        message: &str,
        tools: Vec<serde_json::Value>,
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut tool_calls = Vec::new();
        if let Some(calls) = data["choices"][0]["message"]["tool_calls"].as_array() {
            for call in calls {
                if let (Some(id), Some(name), Some(args)) = (
                    call["id"].as_str(),
                    call["function"]["name"].as_str(),
                    call["function"]["arguments"].as_str(),
                ) {
                    let arguments: HashMap<String, serde_json::Value> =
                        serde_json::from_str(args).unwrap_or_default();
                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        name: name.to_string(),
                        arguments,
                    });
                }
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    async fn chat_anthropic(&self, message: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/messages", self.api_base);
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "max_tokens": 2048,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
        });

        let response = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        data["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::llm_provider("No content in response".to_string()))
    }

    async fn chat_anthropic_with_tools(
        &self,
        message: &str,
        tools: Vec<serde_json::Value>,
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/messages", self.api_base);
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "max_tokens": 2048,
            "tools": tools,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
        });

        let response = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(blocks) = data["content"].as_array() {
            for block in blocks {
                if let Some(text) = block["text"].as_str() {
                    content.push_str(text);
                }
                if block["type"].as_str() == Some("tool_use") {
                    if let (Some(id), Some(name), Some(input)) = (
                        block["id"].as_str(),
                        block["name"].as_str(),
                        block["input"].as_object(),
                    ) {
                        let mut arguments = HashMap::new();
                        for (k, v) in input {
                            arguments.insert(k.clone(), v.clone());
                        }
                        tool_calls.push(ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments,
                        });
                    }
                }
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    async fn chat_ollama(&self, message: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "stream": false,
        });

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::llm_provider("No content in response".to_string()))
    }

    async fn chat_ollama_with_tools(
        &self,
        message: &str,
        _tools: Vec<serde_json::Value>,
    ) -> Result<LlmResponse> {
        // Ollama doesn't natively support tool_calls, so we just return the content
        // without tool calls. Applications can implement tool calling via prompt engineering.
        let content = self.chat_ollama(message).await?;
        Ok(LlmResponse {
            content,
            tool_calls: Vec::new(),
        })
    }

    async fn chat_ollama_with_tools_and_history(
        &self,
        _tools: Vec<serde_json::Value>,
        history: &[serde_json::Value],
    ) -> Result<LlmResponse> {
        // Ollama doesn't natively support tool_calls, so we just return the content
        // without tool calls. Applications can implement tool calling via prompt engineering.
        let client = reqwest::Client::new();
        let url = self.get_chat_endpoint();
        let model = self.normalize_model_name(&self.model);

        let payload = json!({
            "model": model,
            "messages": history,
            "stream": false,
        });

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::http(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::llm_provider(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::serialization(format!("Failed to parse response: {}", e)))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LlmResponse {
            content,
            tool_calls: Vec::new(),
        })
    }
}
