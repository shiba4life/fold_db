// This file is kept for backward compatibility
// It re-exports everything from the web_server module

pub use crate::datafold_node::web_server::types::*;
pub use crate::datafold_node::web_server::handlers::*;
pub use crate::datafold_node::web_server::server::*;

// Re-export specific functions that are used in tests
pub use crate::datafold_node::web_server::types::with_node;
pub use crate::datafold_node::web_server::handlers::schema::handle_schema;
pub use crate::datafold_node::web_server::handlers::schema::handle_execute;
pub use crate::datafold_node::web_server::handlers::schema::handle_delete_schema;
