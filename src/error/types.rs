//! Detailed error types for TakoBull

use std::fmt;

/// Detailed error information for TakoBull operations
#[derive(Debug, Clone)]
pub struct PicoClawError {
    pub code: ErrorCode,
    pub message: String,
    pub context: Option<String>,
}

/// Error codes for different error categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Configuration errors
    ConfigNotFound = 1001,
    ConfigInvalid = 1002,
    ConfigMissing = 1003,

    // Authentication errors
    AuthFailed = 2001,
    TokenExpired = 2002,
    TokenRefreshFailed = 2003,
    PkceInvalid = 2004,

    // Channel errors
    ChannelNotFound = 3001,
    ChannelConnectionFailed = 3002,
    ChannelMessageFailed = 3003,

    // LLM provider errors
    ProviderNotFound = 4001,
    ProviderUnavailable = 4002,
    ProviderRateLimited = 4003,
    ProviderInvalidResponse = 4004,

    // Tool errors
    ToolNotFound = 5001,
    ToolExecutionFailed = 5002,
    ToolTimeout = 5003,

    // Session errors
    SessionNotFound = 6001,
    SessionExpired = 6002,
    SessionPersistenceFailed = 6003,

    // Device errors
    DeviceNotFound = 7001,
    DeviceUnavailable = 7002,
    DeviceOperationFailed = 7003,

    // Internal errors
    InternalError = 9001,
    Unknown = 9999,
}

impl PicoClawError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        PicoClawError {
            code,
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for PicoClawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code as u32, self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = PicoClawError::new(ErrorCode::ConfigNotFound, "config.yaml not found");
        assert_eq!(error.code, ErrorCode::ConfigNotFound);
        assert_eq!(error.message, "config.yaml not found");
        assert!(error.context.is_none());
    }

    #[test]
    fn test_error_with_context() {
        let error = PicoClawError::new(ErrorCode::ConfigInvalid, "invalid config format")
            .with_context("at line 42");
        assert_eq!(error.code, ErrorCode::ConfigInvalid);
        assert_eq!(error.message, "invalid config format");
        assert_eq!(error.context, Some("at line 42".to_string()));
    }

    #[test]
    fn test_error_display_without_context() {
        let error = PicoClawError::new(ErrorCode::AuthFailed, "authentication failed");
        let display_str = format!("{}", error);
        assert!(display_str.contains("2001"));
        assert!(display_str.contains("authentication failed"));
        assert!(!display_str.contains("("));
    }

    #[test]
    fn test_error_display_with_context() {
        let error = PicoClawError::new(ErrorCode::TokenExpired, "token expired")
            .with_context("user_id: 123");
        let display_str = format!("{}", error);
        assert!(display_str.contains("2002"));
        assert!(display_str.contains("token expired"));
        assert!(display_str.contains("user_id: 123"));
    }

    #[test]
    fn test_error_context_capture() {
        let error = PicoClawError::new(ErrorCode::ChannelConnectionFailed, "connection timeout")
            .with_context("telegram channel, attempt 3");
        
        assert!(error.context.is_some());
        assert_eq!(error.context.unwrap(), "telegram channel, attempt 3");
    }

    #[test]
    fn test_error_context_override() {
        let error = PicoClawError::new(ErrorCode::ProviderUnavailable, "provider down")
            .with_context("first context")
            .with_context("second context");
        
        // Last context should override
        assert_eq!(error.context, Some("second context".to_string()));
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::ConfigNotFound as u32, 1001);
        assert_eq!(ErrorCode::AuthFailed as u32, 2001);
        assert_eq!(ErrorCode::ChannelNotFound as u32, 3001);
        assert_eq!(ErrorCode::ProviderNotFound as u32, 4001);
        assert_eq!(ErrorCode::ToolNotFound as u32, 5001);
        assert_eq!(ErrorCode::SessionNotFound as u32, 6001);
        assert_eq!(ErrorCode::DeviceNotFound as u32, 7001);
        assert_eq!(ErrorCode::InternalError as u32, 9001);
        assert_eq!(ErrorCode::Unknown as u32, 9999);
    }

    #[test]
    fn test_error_clone() {
        let error = PicoClawError::new(ErrorCode::ToolExecutionFailed, "tool failed")
            .with_context("tool: shell");
        let cloned = error.clone();
        
        assert_eq!(cloned.code, error.code);
        assert_eq!(cloned.message, error.message);
        assert_eq!(cloned.context, error.context);
    }

    #[test]
    fn test_error_debug_format() {
        let error = PicoClawError::new(ErrorCode::SessionExpired, "session expired")
            .with_context("session_id: abc123");
        let debug_str = format!("{:?}", error);
        
        assert!(debug_str.contains("PicoClawError"));
        assert!(debug_str.contains("SessionExpired"));
        assert!(debug_str.contains("session expired"));
    }
}
