//! Token storage for OAuth2 tokens

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// OAuth2 token pair (access and refresh tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: SystemTime,
}
