# DataFold Node

## Overview
DataFold Node provides an interface to FoldDB's functionality. It lets you load schemas, run queries and mutations, and manage trusted nodes.

## Core Features

### Database Management
- Initialize and load FoldDB instances
- Configure storage locations
- Handle database lifecycle

### Schema Operations
- Load schema definitions
- List and retrieve schemas
- Allow or remove schemas

### Data Operations
- Execute mutations with atomic guarantees
- Perform queries with trust distance control
- Access version history for atom references

### Trusted Nodes
- Manage trusted node identifiers and trust distance

## API Structure

### Initialization
```rust
pub struct DataFoldNode {
    db: FoldDB,
    config: NodeConfig,
    trusted_nodes: HashMap<String, NodeInfo>,
    node_id: String,
}

pub struct NodeConfig {
    storage_path: PathBuf,
    default_trust_distance: u32,
}
```

### Core Methods
```rust
impl DataFoldNode {
    // Initialize a new node
    pub fn new(config: NodeConfig) -> FoldDbResult<Self>;

    // Load an existing database
    pub fn load(config: NodeConfig) -> FoldDbResult<Self>;

    // Schema operations
    pub fn load_schema(&mut self, schema: Schema) -> FoldDbResult<()>;
    pub fn get_schema(&self, schema_id: &str) -> FoldDbResult<Option<Schema>>;
    pub fn list_schemas(&self) -> FoldDbResult<Vec<Schema>>;
    pub fn allow_schema(&mut self, schema_name: &str) -> FoldDbResult<()>;
    pub fn remove_schema(&mut self, schema_name: &str) -> FoldDbResult<()>;

    // Data operations
    pub fn query(&self, query: Query) -> FoldDbResult<Vec<Result<Value, SchemaError>>>;
    pub fn mutate(&mut self, mutation: Mutation) -> FoldDbResult<()>;
    pub fn execute_operation(&mut self, op: Operation) -> FoldDbResult<Value>;
    pub fn get_history(&self, aref_uuid: &str) -> FoldDbResult<Vec<Value>>;

    // Trusted node management
    pub fn add_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()>;
    pub fn remove_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()>;
    pub fn get_trusted_nodes(&self) -> &HashMap<String, NodeInfo>;

    // Node information
    pub fn get_node_id(&self) -> &str;
}
```

## Usage Examples

### Initialize Node
```rust
let config = NodeConfig {
    storage_path: PathBuf::from("./data"),
    default_trust_distance: 1,
};
let mut node = DataFoldNode::new(config)?;
```

### Load Schema
```rust
let schema: Schema = serde_json::from_str(schema_json)?;
node.load_schema(schema)?;
node.allow_schema("user_profile")?;
```

### Execute Query
```rust
let query = Query::new(
    "user_profile".to_string(),
    vec!["name".to_string(), "email".to_string()],
    String::new(), // pub_key
    0,             // use default trust distance
);
let results = node.query(query)?;
```

### Execute Mutation
```rust
let mut fields = HashMap::new();
fields.insert("name".to_string(), json!("John Doe"));
fields.insert("email".to_string(), json!("john@example.com"));
let mutation = Mutation::new(
    "user_profile".to_string(),
    fields,
    String::new(), // pub_key
    0,             // use default trust distance
    MutationType::Create,
);
node.mutate(mutation)?;
```

## Technical Requirements

### Dependencies
- Core FoldDB library
- Serde for serialization
- Standard Rust async runtime

### Constraints
- All operations must maintain atomic guarantees
- Schema must be loaded before any data operations
- Permissions must be validated for all operations
- Trust distance rules must be enforced
- Version history must be maintained

## Error Handling
- Clear error types for different failure scenarios
- Proper propagation of underlying FoldDB errors
- Validation errors for schema and data operations
- Permission-related error cases
- Configuration and initialization errors

## Testing Strategy
- Unit tests for all public methods
- Integration tests for full workflows
- Permission validation tests
- Error handling tests
- Schema validation tests
- Performance benchmarks
