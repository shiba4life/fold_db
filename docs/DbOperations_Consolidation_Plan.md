# DbOperations Consolidation Plan

## Executive Summary

Instead of creating a new data layer, this plan focuses on consolidating all database access to use the existing [`DbOperations`](fold_node/src/db_operations/core.rs:8) infrastructure consistently. The goal is to eliminate legacy patterns, reduce code duplication, and improve the existing unified approach.

## Current State Analysis

### What's Working Well
- [`DbOperations`](fold_node/src/db_operations/core.rs:8) provides a solid unified database abstraction
- Modular operation files ([`schema_operations.rs`](fold_node/src/db_operations/schema_operations.rs:1), [`atom_operations.rs`](fold_node/src/db_operations/atom_operations.rs:1), etc.)
- Consistent error handling within DbOperations
- Cached trees for performance

### Problems to Fix
1. **Mixed Usage Patterns**: [`SchemaCore`](fold_node/src/schema/core.rs:66) conditionally uses DbOperations vs legacy storage
2. **Direct Sled Access**: Some components bypass DbOperations entirely
3. **Inconsistent Error Types**: Different layers use different error types
4. **Code Duplication**: Repetitive serialization patterns across operation modules

## Simplified Improvement Plan

### Phase 1: Complete DbOperations Migration

**Goal**: Ensure all database access goes through [`DbOperations`](fold_node/src/db_operations/core.rs:8)

#### 1.1 Eliminate Legacy SchemaStorage Usage

```rust
// Current problematic pattern in SchemaCore
fn persist_states(&self) -> Result<(), SchemaError> {
    if let Some(_db_ops) = &self.db_ops {
        // Use unified operations
        self.persist_states_unified()
    } else {
        // Use legacy storage - REMOVE THIS
        self.storage.persist_states(&available)
    }
}

// Simplified approach - always use DbOperations
fn persist_states(&self) -> Result<(), SchemaError> {
    self.persist_states_unified()
}
```

#### 1.2 Update SchemaCore Constructor

```rust
// Remove multiple initialization paths, keep only DbOperations version
impl SchemaCore {
    // Remove: init_default(), new(), new_with_trees()
    // Keep only: new_with_db_ops()
    
    pub fn new(path: &str, db_ops: Arc<DbOperations>) -> Result<Self, SchemaError> {
        let schemas_dir = PathBuf::from(path).join("schemas");
        
        Ok(Self {
            schemas: Mutex::new(HashMap::new()),
            available: Mutex::new(HashMap::new()),
            db_ops: Some(db_ops),
            schemas_dir,
        })
    }
}
```

### Phase 2: Enhance DbOperations

**Goal**: Add missing functionality and improve existing operations

#### 2.1 Add Generic Serialization Helpers to DbOperations

```rust
// Enhanced: fold_node/src/db_operations/core.rs
impl DbOperations {
    /// Generic function to store any serializable item in a specific tree
    pub fn store_in_tree<T: Serialize>(
        &self, 
        tree: &sled::Tree, 
        key: &str, 
        item: &T
    ) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(item)
            .map_err(|e| SchemaError::InvalidData(format!("Serialization failed: {}", e)))?;
        
        tree.insert(key.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Store failed: {}", e)))?;
        
        tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Flush failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Generic function to retrieve any deserializable item from a specific tree
    pub fn get_from_tree<T: DeserializeOwned>(
        &self, 
        tree: &sled::Tree, 
        key: &str
    ) -> Result<Option<T>, SchemaError> {
        match tree.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let item = serde_json::from_slice(&bytes)
                    .map_err(|e| SchemaError::InvalidData(format!("Deserialization failed: {}", e)))?;
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SchemaError::InvalidData(format!("Retrieval failed: {}", e))),
        }
    }
    
    /// List all keys in a tree
    pub fn list_keys_in_tree(&self, tree: &sled::Tree) -> Result<Vec<String>, SchemaError> {
        let mut keys = Vec::new();
        for result in tree.iter() {
            let (key, _) = result
                .map_err(|e| SchemaError::InvalidData(format!("Tree iteration failed: {}", e)))?;
            keys.push(String::from_utf8_lossy(&key).to_string());
        }
        Ok(keys)
    }
}
```

#### 2.2 Consolidate Operation Modules

Update all operation modules to use the new generic helpers:

```rust
// Updated: fold_node/src/db_operations/schema_operations.rs
impl DbOperations {
    pub fn store_schema_state(&self, schema_name: &str, state: SchemaState) -> Result<(), SchemaError> {
        self.store_in_tree(&self.schema_states_tree, schema_name, &state)
    }
    
    pub fn get_schema_state(&self, schema_name: &str) -> Result<Option<SchemaState>, SchemaError> {
        self.get_from_tree(&self.schema_states_tree, schema_name)
    }
    
    pub fn store_schema(&self, schema_name: &str, schema: &Schema) -> Result<(), SchemaError> {
        self.store_in_tree(&self.schemas_tree, schema_name, schema)
    }
    
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        self.get_from_tree(&self.schemas_tree, schema_name)
    }
    
    pub fn list_all_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.list_keys_in_tree(&self.schemas_tree)
    }
}
```

### Phase 3: Simplify SchemaCore

**Goal**: Remove complexity and legacy code paths

#### 3.1 Streamlined SchemaCore

```rust
// Simplified: fold_node/src/schema/core.rs
pub struct SchemaCore {
    /// In-memory cache for loaded schemas
    schemas: Mutex<HashMap<String, Schema>>,
    /// All known schemas and their states
    available: Mutex<HashMap<String, (Schema, SchemaState)>>,
    /// Unified database operations
    db_ops: Arc<DbOperations>,
    /// Schema directory path
    schemas_dir: PathBuf,
}

impl SchemaCore {
    /// Single constructor using DbOperations
    pub fn new(path: &str, db_ops: Arc<DbOperations>) -> Result<Self, SchemaError> {
        let schemas_dir = PathBuf::from(path).join("schemas");
        
        if let Err(e) = std::fs::create_dir_all(&schemas_dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to create schemas directory: {}", e
                )));
            }
        }
        
        Ok(Self {
            schemas: Mutex::new(HashMap::new()),
            available: Mutex::new(HashMap::new()),
            db_ops,
            schemas_dir,
        })
    }
    
    /// Simplified state persistence - always use DbOperations
    fn persist_states(&self) -> Result<(), SchemaError> {
        let available = self.available.lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        for (name, (_, state)) in available.iter() {
            self.db_ops.store_schema_state(name, *state)?;
        }
        
        Ok(())
    }
    
    /// Simplified state loading - always use DbOperations
    pub fn load_states(&self) -> HashMap<String, SchemaState> {
        let mut states = HashMap::new();
        
        // Get all schema state keys
        if let Ok(keys) = self.db_ops.list_keys_in_tree(&self.db_ops.schema_states_tree) {
            for key in keys {
                if let Ok(Some(state)) = self.db_ops.get_schema_state(&key) {
                    states.insert(key, state);
                }
            }
        }
        
        states
    }
    
    /// Simplified schema persistence - always use DbOperations
    fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        self.db_ops.store_schema(&schema.name, schema)
    }
}
```

### Phase 4: Update FoldDB Integration

**Goal**: Ensure FoldDB uses DbOperations consistently

#### 4.1 Simplified FoldDB Constructor

```rust
// Updated: fold_node/src/fold_db_core/mod.rs
impl FoldDB {
    pub fn new(path: &str) -> sled::Result<Self> {
        let db = sled::open(path)?;
        let db_ops = Arc::new(DbOperations::new(db.clone())
            .map_err(|e| sled::Error::Unsupported(e.to_string()))?);
        
        let atom_manager = AtomManager::new(db_ops.clone());
        let field_manager = FieldManager::new(atom_manager.clone());
        let collection_manager = CollectionManager::new(field_manager.clone());
        
        // Always use DbOperations for SchemaCore
        let schema_manager = Arc::new(
            SchemaCore::new(path, db_ops.clone())
                .map_err(|e| sled::Error::Unsupported(e.to_string()))?
        );
        
        // ... rest of initialization
        
        Ok(Self {
            atom_manager,
            field_manager,
            collection_manager,
            schema_manager,
            transform_manager,
            transform_orchestrator,
            db_ops,
            permission_wrapper: PermissionWrapper::new(),
        })
    }
}
```

### Phase 5: Remove Legacy Code

**Goal**: Clean up unused legacy patterns

#### 5.1 Files to Remove/Simplify

1. **Remove**: [`SchemaStorage`](fold_node/src/schema/storage.rs:11) - functionality moved to DbOperations
2. **Simplify**: [`SchemaCore`](fold_node/src/schema/core.rs:66) - remove conditional logic
3. **Update**: All direct sled access to use DbOperations

#### 5.2 Error Handling Consolidation

```rust
// Convert SchemaError to use DbOperations error patterns
impl From<sled::Error> for SchemaError {
    fn from(err: sled::Error) -> Self {
        SchemaError::InvalidData(format!("Database error: {}", err))
    }
}
```

## Implementation Benefits

### 1. Simplified Architecture
- Single database access pattern through DbOperations
- Eliminated conditional legacy/unified code paths
- Consistent error handling

### 2. Reduced Code Duplication
- Generic serialization helpers in DbOperations
- Reusable tree operations
- Consolidated error handling

### 3. Better Maintainability
- Single source of truth for database operations
- Easier to debug and test
- Clear separation of concerns

### 4. Performance Improvements
- Cached trees in DbOperations
- Reduced lock contention through simplified patterns
- More efficient serialization

## Migration Steps

### Week 1: DbOperations Enhancement
1. Add generic serialization helpers to DbOperations
2. Update all operation modules to use new helpers
3. Test enhanced DbOperations functionality

### Week 2: SchemaCore Migration
1. Remove conditional legacy/unified code paths
2. Update SchemaCore to always use DbOperations
3. Remove SchemaStorage dependency

### Week 3: FoldDB Integration
1. Update FoldDB to use simplified SchemaCore
2. Ensure all database access goes through DbOperations
3. Remove direct sled access patterns

### Week 4: Cleanup and Testing
1. Remove SchemaStorage and related legacy code
2. Comprehensive testing of unified approach
3. Performance validation

## Success Metrics

- **Code Reduction**: 30-40% reduction in database-related code
- **Consistency**: 100% of database access through DbOperations
- **Performance**: Maintain or improve current performance
- **Maintainability**: Single pattern for all database operations

## Risk Mitigation

- **Incremental Changes**: Update one component at a time
- **Comprehensive Testing**: Test each migration step
- **Rollback Plan**: Keep legacy code until validation complete
- **Performance Monitoring**: Benchmark throughout migration

This simplified approach leverages the existing DbOperations infrastructure while eliminating complexity and duplication. It's much more practical than creating a new data layer and will deliver the same benefits with less risk and effort.