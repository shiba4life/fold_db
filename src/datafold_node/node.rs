use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;

use crate::folddb::FoldDB;
use crate::schema::types::{Mutation, Query};
use crate::schema::{Schema, SchemaError};
use crate::datafold_node::{
    config::NodeConfig,
    docker::{self, ContainerState, ContainerStatus},
    error::{NodeError, NodeResult},
};

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
        docker::check_docker_available()?;

        let container_id = docker::create_container(image, &self.config.docker)?;
        
        if let Err(e) = docker::start_container(&container_id) {
            // Cleanup failed container
            let _ = docker::remove_container(&container_id);
            return Err(e);
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
            docker::stop_container(&container.id)?;
            docker::remove_container(&container.id)?;

            // Cleanup network if isolated
            if let Some(network_id) = container.network_id {
                docker::cleanup_network(&network_id);
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
