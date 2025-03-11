pub mod schema;
pub mod network;
pub mod app;
pub mod auth;
pub mod schema_loader;

// Re-export all handlers for easier imports
pub use schema::*;
pub use network::*;
pub use app::*;
pub use auth::*;
pub use schema_loader::*;
