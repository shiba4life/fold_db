//! Authentication functionality for DataFold node

pub mod signature_auth;

#[cfg(test)]
pub mod signature_auth_tests;

pub use signature_auth::{
    SignatureAuthConfig, SignatureVerificationState, 
    SignatureVerificationMiddleware, AuthenticationError
};