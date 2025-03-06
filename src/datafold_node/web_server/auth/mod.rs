pub mod web_auth_manager;
pub mod key_verification;

// Re-export key types and functions for easier imports
pub use web_auth_manager::{WebAuthManager, WebAuthConfig, PublicKey, Signature, WebRequest};
pub use key_verification::{KeyVerificationMiddleware, with_auth, AuthenticatedRequest, AuthHeaders};
