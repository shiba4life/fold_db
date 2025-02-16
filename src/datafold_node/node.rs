use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;

use crate::folddb::FoldDB;
use crate::schema::types::{Mutation, Query};
use crate::schema::{Schema, SchemaError};
use crate::schema_interpreter::types::Operation;
use crate::datafold_node::{
    config::NodeConfig,
    docker::{self, ContainerState, ContainerStatus},
    error::{NodeError, NodeResult},
};
use crate::datafold_node::config::NodeInfo;

/// A node in the FoldDB distributed database system.
/// 
/// DataFoldNode is responsible for:
/// - Managing local data storage and processing
/// - Running and supervising application containers
/// - Controlling network access and isolation
/// - Handling schema operations
/// - Processing queries and mutations
/// - Maintaining trust relationships
/// - Managing version history
/// 
/// The node provides a secure environment for applications by:
/// - Running them in isolated Docker containers
/// - Mediating all network access
/// - Enforcing schema-based data validation
/// - Applying permission and payment policies
#[derive(Clone)]
pub struct DataFoldNode {
    /// The underlying database instance for data storage and operations
    db: Arc<FoldDB>,
    /// Configuration settings for this node
    config: NodeConfig,
    /// Map of active application containers and their states
    containers: HashMap<String, ContainerState>,
    /// Map of trusted nodes and their trust distances
    trusted_nodes: HashMap<String, NodeInfo>,
}

impl DataFoldNode {
    /// Creates a new DataFoldNode with the specified configuration.
    /// 
    /// This method:
    /// 1. Initializes a new database instance at the storage path
    /// 2. Sets up Docker networking if container support is enabled
    /// 3. Configures trust relationships and permissions
    /// 
    /// If the storage path already contains a database, a new node will be created
    /// that can access that data.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Configuration for the new node
    /// 
    /// # Returns
    /// 
    /// A Result containing the new node or an error if initialization failed
    /// 
    /// # Example
    /// 
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
            trusted_nodes: HashMap::new(),
        })
    }

    /// Loads a Docker application into a new container.
    /// 
    /// This method:
    /// 1. Verifies Docker availability
    /// 2. Creates a new container from the specified image
    /// 3. Starts the container
    /// 4. Tracks the container's state
    /// 
    /// If any step fails, the container is cleaned up before returning an error.
    /// 
    /// # Arguments
    /// 
    /// * `image` - Docker image name to run
    /// * `app_id` - Unique identifier for the application
    /// 
    /// # Returns
    /// 
    /// A Result containing the container ID or an error
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
    /// This method:
    /// 1. Stops the container if running
    /// 2. Removes the container
    /// 3. Cleans up any associated network configuration
    /// 4. Updates internal container tracking
    /// 
    /// # Arguments
    /// 
    /// * `app_id` - Application identifier
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or an error
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
    /// 
    /// * `app_id` - Application identifier
    /// 
    /// # Returns
    /// 
    /// A Result containing the container status if found
    pub fn get_docker_app_status(&self, app_id: &str) -> NodeResult<Option<ContainerStatus>> {
        Ok(self.containers.get(app_id).map(|c| c.status.clone()))
    }

    /// Loads an existing database node from the specified configuration.
    /// 
    /// Currently behaves the same as `new()` since FoldDB automatically handles
    /// existing data at the storage path.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Configuration pointing to the existing database location
    /// 
    /// # Returns
    /// 
    /// A Result containing the loaded node or an error
    pub fn load(config: NodeConfig) -> NodeResult<Self> {
        // For now, loading is same as creating new since FoldDB handles existing data
        Self::new(config)
    }

    /// Loads a schema into the database.
    pub fn load_schema(&mut self, schema: Schema) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .load_schema(schema)
            .map_err(NodeError::from)?;
        Ok(())
    }

    /// Executes an operation (query or mutation) on the database.
    pub fn execute_operation(&mut self, operation: Operation) -> NodeResult<Value> {
        match operation {
            Operation::Query { schema, fields, filter: _ } => {
                let query = Query {
                    schema_name: schema,
                    fields,
                    pub_key: String::new(), // TODO: Get from auth context
                    trust_distance: self.config.default_trust_distance,
                };
                
                let results = self.db.query_schema(query);
                Ok(serde_json::to_value(&results)
                    .map_err(|e| NodeError::ConfigError(e.to_string()))?)
            },
            Operation::Mutation { schema, operation: _, data } => {
                let fields_and_values = match data {
                      Value::Object(map) => map.into_iter()
                        .collect(),
                    _ => return Err(NodeError::ConfigError("Mutation data must be an object".into()))
                };

                let mutation = Mutation {
                    schema_name: schema,
                    fields_and_values,
                    pub_key: String::new(), // TODO: Get from auth context
                    trust_distance: self.config.default_trust_distance,
                };

                Arc::get_mut(&mut self.db)
                    .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
                    .write_schema(mutation)
                    .map_err(NodeError::from)?;

                Ok(Value::Null)
            }
        }
    }

    /// Retrieves a schema by its ID.
    /// 
    /// # Arguments
    /// 
    /// * `schema_id` - The unique identifier of the schema
    /// 
    /// # Returns
    /// 
    /// A Result containing the schema if found
    pub fn get_schema(&self, schema_id: &str) -> NodeResult<Option<Schema>> {
        Ok(self.db.schema_manager.get_schema(schema_id)?)
    }

    /// Executes a query against the database.
    /// 
    /// This method:
    /// 1. Applies default trust distance if not specified
    /// 2. Validates permissions for each requested field
    /// 3. Retrieves and returns the requested data
    /// 
    /// # Arguments
    /// 
    /// * `query` - The query to execute
    /// 
    /// # Returns
    /// 
    /// A Result containing query results or errors for each field
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
    /// This method:
    /// 1. Validates the mutation against schema constraints
    /// 2. Checks permissions for each field
    /// 3. Creates new versions of modified data
    /// 4. Updates references to point to new versions
    /// 
    /// # Arguments
    /// 
    /// * `mutation` - The mutation to execute
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or an error
    /// 
    /// # Errors
    /// 
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

    /// Adds a trusted node to the node's trusted nodes list.
    /// 
    /// Trust relationships affect:
    /// - Permission calculations
    /// - Payment requirements
    /// - Data access control
    /// 
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to add
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or an error
    pub fn add_trusted_node(&mut self, node_id: &str) -> NodeResult<()> {
        self.trusted_nodes.insert(node_id.to_string(), NodeInfo {
            id: node_id.to_string(),
            trust_distance: self.config.default_trust_distance,
        });
        Ok(())
    }

    /// Removes a trusted node from the node's trusted nodes list.
    /// 
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to remove
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or an error
    pub fn remove_trusted_node(&mut self, node_id: &str) -> NodeResult<()> {
        self.trusted_nodes.remove(node_id);
        Ok(())
    }

    /// Gets the current list of trusted nodes and their trust distances.
    /// 
    /// # Returns
    /// 
    /// A reference to the map of trusted nodes
    pub fn get_trusted_nodes(&self) -> &HashMap<String, NodeInfo> {
        &self.trusted_nodes
    }
    
    /// Retrieves the version history for a specific atom reference.
    /// 
    /// This method follows the chain of previous versions to build
    /// a complete history of changes to the data.
    /// 
    /// # Arguments
    /// 
    /// * `aref_uuid` - The UUID of the atom reference
    /// 
    /// # Returns
    /// 
    /// A Result containing the list of historical versions
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
    /// This method enables queries and mutations on the specified schema
    /// after it has been loaded and validated.
    /// 
    /// # Arguments
    /// 
    /// * `schema_name` - Name of the schema to allow operations on
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or an error
    pub fn allow_schema(&mut self, schema_name: &str) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .allow_schema(schema_name)?;
        Ok(())
    }
}
