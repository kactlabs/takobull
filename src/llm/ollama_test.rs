//! Tests for Ollama provider integration

#[cfg(test)]
mod tests {
    use crate::llm::client::LlmClient;

    #[test]
    fn test_ollama_client_creation() {
        let client = LlmClient::new(
            "ollama",
            "llama2",
            "",
            "http://localhost:11434",
        );
        
        // Verify client is created with correct provider
        assert_eq!(client.provider, "ollama");
        assert_eq!(client.model, "llama2");
        assert_eq!(client.api_base, "http://localhost:11434");
    }

    #[test]
    fn test_model_name_normalization() {
        let client = LlmClient::new(
            "ollama",
            "ollama/llama2",
            "",
            "http://localhost:11434",
        );
        
        let normalized = client.normalize_model_name("ollama/llama2");
        assert_eq!(normalized, "llama2");
    }

    #[test]
    fn test_vllm_model_name_normalization() {
        let client = LlmClient::new(
            "vllm",
            "vllm/meta-llama/Llama-2-7b-hf",
            "test-key",
            "http://localhost:8000",
        );
        
        let normalized = client.normalize_model_name("vllm/meta-llama/Llama-2-7b-hf");
        assert_eq!(normalized, "meta-llama/Llama-2-7b-hf");
    }

    #[test]
    fn test_model_name_without_prefix() {
        let client = LlmClient::new(
            "ollama",
            "llama2",
            "",
            "http://localhost:11434",
        );
        
        let normalized = client.normalize_model_name("llama2");
        assert_eq!(normalized, "llama2");
    }
}
