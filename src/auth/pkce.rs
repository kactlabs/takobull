//! PKCE (Proof Key for Public Clients) implementation

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;

/// PKCE challenge and verifier pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge with a cryptographically secure code verifier
    ///
    /// # Requirements
    /// - Requirement 4.1: Implements PKCE challenge generation
    /// - Requirement 4.2: Generates cryptographically secure code verifier
    pub fn generate() -> Self {
        // Generate a random code verifier (43-128 characters, URL-safe base64)
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let code_verifier = URL_SAFE_NO_PAD.encode(&random_bytes);

        // Generate code challenge as SHA256 hash of verifier
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(&hash);

        PkceChallenge {
            code_verifier,
            code_challenge,
        }
    }

    /// Verify that a code verifier matches a code challenge
    pub fn verify(&self, challenge: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.code_verifier.as_bytes());
        let hash = hasher.finalize();
        let computed_challenge = URL_SAFE_NO_PAD.encode(&hash);
        computed_challenge == challenge
    }

    /// Check if the code verifier is valid (43-128 characters, URL-safe base64)
    pub fn is_valid_verifier(&self) -> bool {
        let len = self.code_verifier.len();
        len >= 43 && len <= 128 && self.code_verifier.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_'
        })
    }

    /// Check if the code challenge is valid (URL-safe base64)
    pub fn is_valid_challenge(&self) -> bool {
        self.code_challenge.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_'
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_challenge_generation() {
        let challenge = PkceChallenge::generate();
        assert!(!challenge.code_verifier.is_empty());
        assert!(!challenge.code_challenge.is_empty());
    }

    #[test]
    fn test_pkce_verifier_length() {
        let challenge = PkceChallenge::generate();
        let len = challenge.code_verifier.len();
        assert!(len >= 43 && len <= 128);
    }

    #[test]
    fn test_pkce_verifier_is_url_safe_base64() {
        let challenge = PkceChallenge::generate();
        assert!(challenge.code_verifier.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_'
        }));
    }

    #[test]
    fn test_pkce_challenge_is_url_safe_base64() {
        let challenge = PkceChallenge::generate();
        assert!(challenge.code_challenge.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_'
        }));
    }

    #[test]
    fn test_pkce_verify_valid() {
        let challenge = PkceChallenge::generate();
        assert!(challenge.verify(&challenge.code_challenge));
    }

    #[test]
    fn test_pkce_verify_invalid() {
        let challenge = PkceChallenge::generate();
        assert!(!challenge.verify("invalid_challenge"));
    }

    #[test]
    fn test_pkce_is_valid_verifier() {
        let challenge = PkceChallenge::generate();
        assert!(challenge.is_valid_verifier());
    }

    #[test]
    fn test_pkce_is_valid_challenge() {
        let challenge = PkceChallenge::generate();
        assert!(challenge.is_valid_challenge());
    }

    #[test]
    fn test_pkce_multiple_generations_are_different() {
        let challenge1 = PkceChallenge::generate();
        let challenge2 = PkceChallenge::generate();
        assert_ne!(challenge1.code_verifier, challenge2.code_verifier);
        assert_ne!(challenge1.code_challenge, challenge2.code_challenge);
    }
}
