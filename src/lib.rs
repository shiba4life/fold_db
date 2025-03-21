pub mod atom;
pub mod db_operations;
pub mod error;
pub mod fees;
pub mod permissions;
pub mod schema;
pub mod datafold_node;
pub mod fold_db_core;
pub mod testing;

// Re-export main types for convenience
pub use datafold_node::DataFoldNode;
pub use datafold_node::config::NodeConfig;
pub use datafold_node::loader::load_schema_from_file;
pub use error::{FoldDbError, FoldDbResult};
pub use fold_db_core::FoldDB;

// Re-export schema types needed for CLI
pub use schema::Schema;
pub use schema::types::operations::MutationType;
pub use schema::types::operation::Operation;
