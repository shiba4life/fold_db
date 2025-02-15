//! DataFoldNode module provides the core node functionality for the FoldDB database system.
//! 
//! A DataFoldNode represents a single node in the distributed database system, responsible for:
//! - Managing local data storage
//! - Handling schema operations
//! - Processing queries and mutations
//! - Managing trust relationships between nodes
//! 
//! # Example
//! ```no_run
//! use fold_db::{DataFoldNode, NodeConfig, datafold_node::DockerConfig};
//! use std::path::PathBuf;
//! 
//! let config = NodeConfig {
//!     storage_path: PathBuf::from("/tmp/db"),
//!     default_trust_distance: 1,
//!     docker: DockerConfig::default(),
//! };
//! 
//! let node = DataFoldNode::new(config).expect("Failed to create node");
//! ```

use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

use crate::folddb::FoldDB;
use crate::schema::types::{Mutation, Query};
use crate::schema::{Schema, SchemaError};

// Docker container configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DockerConfig {
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// CPU limit (number of cores)
    pub cpu_limit: f64,
    /// Container environment variables
    pub environment: HashMap<String, String>,
    /// Network configuration
    pub network_config: DockerNetworkConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DockerNetworkConfig {
    /// Enable network isolation
    pub network_isolated: bool,
    /// Exposed ports configuration
    pub exposed_ports: HashMap<u16, u16>, // container_port -> host_port
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            memory_limit: 512 * 1024 * 1024, // 512MB
            cpu_limit: 1.0,
            environment: HashMap::new(),
            network_config: DockerNetworkConfig {
                network_isolated: true,
                exposed_ports: HashMap::new(),
            },
        }
    }
}

/// Configuration for a DataFoldNode instance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    /// Path where the node will store its data
    pub storage_path: PathBuf,
    /// Default trust distance for queries when not explicitly specified
    /// Must be greater than 0
    pub default_trust_distance: u32,
    /// Docker configuration for containerized applications
    #[serde(default)]
    pub docker: DockerConfig,
}

use std::process::Command;

/// Container state tracking
#[derive(Debug, Clone)]
pub struct ContainerState {
    /// Container ID
    pub id: String,
    /// Container status
    pub status: ContainerStatus,
    /// Network ID if using isolated network
    pub network_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Failed(String),
}

/// A node in the FoldDB distributed database system.
/// 
/// DataFoldNode manages local data storage and processing, including:
/// - Schema management
/// - Query processing
/// - Mutation handling
/// - Version history tracking
#[derive(Clone)]
pub struct DataFoldNode {
    /// The underlying database instance
    db: Arc<FoldDB>,
    /// Node configuration
    config: NodeConfig,
    /// Active containers
    containers: HashMap<String, ContainerState>,
}

/// Errors that can occur during node operations.
#[derive(Debug)]
pub enum NodeError {
    /// Error occurred in the underlying database
    DatabaseError(String),
    /// Error related to schema operations
    SchemaError(SchemaError),
    /// Error related to insufficient permissions
    PermissionError(String),
    /// Error in node configuration
    ConfigError(String),
    /// Error in Docker operations
    DockerError(String),
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::SchemaError(err) => write!(f, "Schema error: {}", err),
            Self::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::DockerError(msg) => write!(f, "Docker error: {}", msg),
        }
    }
}

impl std::error::Error for NodeError {}

impl From<SchemaError> for NodeError {
    fn from(error: SchemaError) -> Self {
        NodeError::SchemaError(error)
    }
}

impl From<sled::Error> for NodeError {
    fn from(error: sled::Error) -> Self {
        NodeError::DatabaseError(error.to_string())
    }
}

pub type NodeResult<T> = Result<T, NodeError>;

impl DataFoldNode {
    /// Creates a new DataFoldNode with the specified configuration.
    /// 
    /// This will initialize a new database instance at the specified storage path
    /// and set up Docker networking if container support is enabled.
    /// 
    /// This will initialize a new database instance at the specified storage path.
    /// If the storage path already contains a database, a new node will be created
    /// that can access that data.
    /// 
    /// # Arguments
    /// * `config` - Configuration for the new node
    /// 
    /// # Returns
    /// * `NodeResult<Self>` - The newly created node or an error if initialization failed
    /// 
    /// # Example
    /// ```no_run
    /// use fold_db::{DataFoldNode, NodeConfig, datafold_node::DockerConfig};
    /// use std::path::PathBuf;
    /// 
    /// let config = NodeConfig {
    ///     storage_path: PathBuf::from("/tmp/db"),
    ///     default_trust_distance: 1,
    ///     docker: DockerConfig::default(),
    /// };
    /// 
    /// let node = DataFoldNode::new(config).expect("Failed to create node");
    /// ```
    pub fn new(config: NodeConfig) -> NodeResult<Self> {
        let db = Arc::new(FoldDB::new(
            config
                .storage_path
                .to_str()
                .ok_or_else(|| NodeError::ConfigError("Invalid storage path".to_string()))?,
        )?);

        Ok(Self { 
            db, 
            config,
            containers: HashMap::new(),
        })
    }

    /// Loads a Docker application into a new container.
    /// 
    /// # Arguments
    /// * `image` - Docker image name
    /// * `app_id` - Unique identifier for the application
    /// 
    /// # Returns
    /// * `NodeResult<String>` - Container ID if successful
    pub fn load_docker_app(&mut self, image: &str, app_id: &str) -> NodeResult<String> {
        // Check if Docker is available
        if let Err(e) = Command::new("docker").arg("--version").output() {
            return Err(NodeError::DockerError(format!("Docker not available: {}", e)));
        }

        // Create container with resource limits
        let create_output = Command::new("docker")
            .arg("create")
            .arg("--memory")
            .arg(format!("{}b", self.config.docker.memory_limit))
            .arg("--cpus")
            .arg(self.config.docker.cpu_limit.to_string())
            .arg("--network")
            .arg(if self.config.docker.network_config.network_isolated { "none" } else { "bridge" })
            .args(self.config.docker.environment.iter().flat_map(|(k, v)| {
                let env_arg = format!("{}={}", k, v);
                vec!["-e".to_string(), env_arg]
            }).collect::<Vec<String>>())
            .args(self.config.docker.network_config.exposed_ports.iter().flat_map(|(container, host)| {
                let port_arg = format!("{}:{}", host, container);
                vec!["-p".to_string(), port_arg]
            }).collect::<Vec<String>>())
            .arg(image)
            .output()
            .map_err(|e| NodeError::DockerError(format!("Failed to create container: {}", e)))?;

        if !create_output.status.success() {
            return Err(NodeError::DockerError(format!(
                "Container creation failed: {}",
                String::from_utf8_lossy(&create_output.stderr)
            )));
        }

        let container_id = String::from_utf8_lossy(&create_output.stdout).trim().to_string();

        // Start container
        let start_output = Command::new("docker")
            .args(["start", &container_id])
            .output()
            .map_err(|e| NodeError::DockerError(format!("Failed to start container: {}", e)))?;

        if !start_output.status.success() {
            // Cleanup failed container
            let _ = Command::new("docker")
                .args(["rm", &container_id])
                .output();
            
            return Err(NodeError::DockerError(format!(
                "Container start failed: {}",
                String::from_utf8_lossy(&start_output.stderr)
            )));
        }

        // Track container state
        self.containers.insert(app_id.to_string(), ContainerState {
            id: container_id.clone(),
            status: ContainerStatus::Running,
            network_id: None,
        });

        Ok(container_id)
    }

    /// Stops and removes a Docker application container.
    /// 
    /// # Arguments
    /// * `app_id` - Application identifier
    /// 
    /// # Returns
    /// * `NodeResult<()>` - Success or error
    pub fn remove_docker_app(&mut self, app_id: &str) -> NodeResult<()> {
        if let Some(container) = self.containers.remove(app_id) {
            // Stop container
            let stop_output = Command::new("docker")
                .args(["stop", &container.id])
                .output()
                .map_err(|e| NodeError::DockerError(format!("Failed to stop container: {}", e)))?;

            if !stop_output.status.success() {
                return Err(NodeError::DockerError(format!(
                    "Container stop failed: {}",
                    String::from_utf8_lossy(&stop_output.stderr)
                )));
            }

            // Remove container
            let rm_output = Command::new("docker")
                .args(["rm", &container.id])
                .output()
                .map_err(|e| NodeError::DockerError(format!("Failed to remove container: {}", e)))?;

            if !rm_output.status.success() {
                return Err(NodeError::DockerError(format!(
                    "Container removal failed: {}",
                    String::from_utf8_lossy(&rm_output.stderr)
                )));
            }

            // Cleanup network if isolated
            if let Some(network_id) = container.network_id {
                let _ = Command::new("docker")
                    .args(["network", "rm", &network_id])
                    .output();
            }
        }

        Ok(())
    }

    /// Gets the status of a Docker application container.
    /// 
    /// # Arguments
    /// * `app_id` - Application identifier
    /// 
    /// # Returns
    /// * `NodeResult<Option<ContainerStatus>>` - Container status if found
    pub fn get_docker_app_status(&self, app_id: &str) -> NodeResult<Option<ContainerStatus>> {
        Ok(self.containers.get(app_id).map(|c| c.status.clone()))
    }

    /// Loads an existing database node from the specified configuration.
    /// 
    /// Currently behaves the same as `new()` since FoldDB automatically handles
    /// existing data at the storage path.
    /// 
    /// # Arguments
    /// * `config` - Configuration pointing to the existing database location
    /// 
    /// # Returns
    /// * `NodeResult<Self>` - The loaded node or an error if loading failed
    pub fn load(config: NodeConfig) -> NodeResult<Self> {
        // For now, loading is same as creating new since FoldDB handles existing data
        Self::new(config)
    }

    /// Loads a schema into the database.
    /// 
    /// The schema will be available for subsequent queries and mutations.
    /// 
    /// # Arguments
    /// * `schema` - The schema to load
    /// 
    /// # Returns
    /// * `NodeResult<()>` - Success or an error if schema loading failed
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The schema is invalid
    /// - There are conflicts with existing schemas
    /// - The database is currently locked
    pub fn load_schema(&mut self, schema: Schema) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .load_schema(schema)?;
        Ok(())
    }

    /// Retrieves a schema by its ID.
    /// 
    /// # Arguments
    /// * `schema_id` - The unique identifier of the schema
    /// 
    /// # Returns
    /// * `NodeResult<Option<Schema>>` - The schema if found, None if not found, or an error
    pub fn get_schema(&self, schema_id: &str) -> NodeResult<Option<Schema>> {
        Ok(self.db.schema_manager.get_schema(schema_id)?)
    }

    /// Executes a query against the database.
    /// 
    /// If the query's trust_distance is 0, it will be set to the node's default_trust_distance.
    /// 
    /// # Arguments
    /// * `query` - The query to execute
    /// 
    /// # Returns
    /// * `NodeResult<Vec<Result<Value, SchemaError>>>` - Query results or errors for each matched item
    pub fn query(&self, mut query: Query) -> NodeResult<Vec<Result<Value, SchemaError>>> {
        // Apply default trust distance if not set
        if query.trust_distance == 0 {
            query.trust_distance = self.config.default_trust_distance;
        }

        // Execute query and return results
        Ok(self.db.query_schema(query))
    }

    /// Executes a mutation on the database.
    /// 
    /// # Arguments
    /// * `mutation` - The mutation to execute
    /// 
    /// # Returns
    /// * `NodeResult<()>` - Success or an error if the mutation failed
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The mutation violates schema constraints
    /// - The database is locked
    /// - Insufficient permissions
    pub fn mutate(&mut self, mutation: Mutation) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .write_schema(mutation)?;
        Ok(())
    }

    /// Sets the default trust distance for this node.
    /// 
    /// The trust distance determines how far queries will traverse the network
    /// when not explicitly specified in the query.
    /// 
    /// # Arguments
    /// * `distance` - The new default trust distance (must be > 0)
    /// 
    /// # Returns
    /// * `NodeResult<()>` - Success or an error if the distance is invalid
    pub fn set_trust_distance(&mut self, distance: u32) -> NodeResult<()> {
        if distance == 0 {
            return Err(NodeError::ConfigError(
                "Trust distance must be greater than 0".to_string(),
            ));
        }
        self.config.default_trust_distance = distance;
        Ok(())
    }

    /// Retrieves the version history for a specific atom reference.
    /// 
    /// Returns all historical versions of the specified atom in chronological order.
    /// 
    /// # Arguments
    /// * `aref_uuid` - The UUID of the atom reference
    /// 
    /// # Returns
    /// * `NodeResult<Vec<Value>>` - List of historical versions or an error
    pub fn get_history(&self, aref_uuid: &str) -> NodeResult<Vec<Value>> {
        let history = self
            .db
            .get_atom_history(aref_uuid)
            .map_err(|e| NodeError::DatabaseError(e.to_string()))?;

        Ok(history
            .into_iter()
            .map(|atom| atom.content().clone())
            .collect())
    }

    /// Allows operations on a schema.
    /// 
    /// Grants permission to perform operations on the specified schema.
    /// 
    /// # Arguments
    /// * `schema_name` - Name of the schema to allow operations on
    /// 
    /// # Returns
    /// * `NodeResult<()>` - Success or an error if permission cannot be granted
    pub fn allow_schema(&mut self, schema_name: &str) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .allow_schema(schema_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_config() -> NodeConfig {
        let dir = tempdir().unwrap();
        NodeConfig {
            storage_path: dir.path().to_path_buf(),
            default_trust_distance: 1,
            docker: DockerConfig::default(),
        }
    }

    #[test]
    fn test_node_creation() {
        let config = create_test_config();
        let node = DataFoldNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_trust_distance_validation() {
        let config = create_test_config();
        let mut node = DataFoldNode::new(config).unwrap();

        assert!(node.set_trust_distance(0).is_err());
        assert!(node.set_trust_distance(1).is_ok());
    }

    #[test]
    fn test_docker_app_lifecycle() {
        // Skip if Docker is not available
        if Command::new("docker").arg("--version").output().is_err() {
            println!("Skipping docker test - Docker not available");
            return;
        }

        let mut config = create_test_config();
        
        // Configure Docker settings
        config.docker.memory_limit = 256 * 1024 * 1024; // 256MB
        config.docker.cpu_limit = 0.5;
        config.docker.environment.insert("TEST_ENV".to_string(), "test_value".to_string());
        config.docker.network_config.exposed_ports.insert(8080, 8081);

        let mut node = DataFoldNode::new(config).unwrap();

        // Test loading app
        let app_id = "test-app";
        let container_id = node.load_docker_app("hello-world:latest", app_id).unwrap();
        assert!(!container_id.is_empty());

        // Test getting status
        let status = node.get_docker_app_status(app_id).unwrap();
        assert!(matches!(status, Some(ContainerStatus::Running)));

        // Test removing app
        assert!(node.remove_docker_app(app_id).is_ok());

        // Verify app is removed
        let status = node.get_docker_app_status(app_id).unwrap();
        assert!(status.is_none());
    }

    #[test]
    fn test_docker_app_error_handling() {
        // Skip if Docker is not available
        if Command::new("docker").arg("--version").output().is_err() {
            println!("Skipping docker test - Docker not available");
            return;
        }

        let config = create_test_config();
        let mut node = DataFoldNode::new(config).unwrap();

        // Test loading non-existent image
        let result = node.load_docker_app("non-existent-image:latest", "test-app");
        assert!(matches!(result, Err(NodeError::DockerError(_))));

        // Test removing non-existent app
        let result = node.remove_docker_app("non-existent-app");
        assert!(result.is_ok()); // Should succeed silently if app doesn't exist
    }
}
