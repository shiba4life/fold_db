# PBI-5: Runtime System Updates for Composable AtomRef Operations

## Overview

This PBI updates all runtime systems to use pre-created composable AtomRefs instead of creating them at runtime, ensuring seamless field operations with the new composable type system while maintaining backward compatibility.

[View in Backlog](../backlog.md#user-content-5)

## Problem Statement

Current runtime systems create AtomRefs during field operations and lack support for composable AtomRef types. This approach creates inconsistent state and doesn't leverage the benefits of pre-created ARefs from schema approval. The system needs to be updated to use existing composable ARefs for all field operations.

## User Stories

### Primary User Story
As a developer, I want runtime systems updated to use pre-created composable ARefs, so that field operations work seamlessly with the new composable type system.

### Supporting User Stories
- As a system operator, I want field operations to use pre-created ARefs, so that state management is consistent and predictable
- As a developer, I want all composable ARef operations supported, so that I can work with hierarchical data structures
- As an API user, I want HTTP endpoints to support composable field types, so that I can interact with complex schemas
- As a developer, I want comprehensive error handling, so that missing or invalid ARefs are handled gracefully

## Technical Approach

### Field Processing Updates

#### Enhanced Field Value Processing
```rust
impl AtomManager {
    /// Handle field value set request using pre-created ARefs
    pub fn handle_field_value_set(&self, request: FieldValueSetRequest) -> Result<String, Box<dyn std::error::Error>> {
        // Get ARef UUID for this field from schema mappings
        let aref_uuid = self.get_field_aref_uuid(&request.schema_name, &request.field_name)?;
        
        // Retrieve the composable ARef
        let composable_aref = self.get_composable_aref(&aref_uuid)?;
        
        // Process the value based on ARef type
        match composable_aref {
            ComposableAtomRef::Single(container) => {
                self.handle_single_container_operation(container, &request)?;
            }
            ComposableAtomRef::Composed { container, element_type, element_config } => {
                self.handle_composed_operation(container, element_type, element_config, &request)?;
            }
        }
        
        Ok(aref_uuid)
    }
    
    /// Handle single container operations
    fn handle_single_container_operation(
        &self,
        container: &AtomRefContainer,
        request: &FieldValueSetRequest
    ) -> Result<(), Box<dyn std::error::Error>> {
        match container {
            AtomRefContainer::Single(aref) => {
                self.handle_single_aref_operation(aref, request)
            }
            AtomRefContainer::Range(range) => {
                self.handle_range_aref_operation(range, request)
            }
            AtomRefContainer::Hash(hash) => {
                self.handle_hash_aref_operation(hash, request)
            }
            AtomRefContainer::Collection(collection) => {
                self.handle_collection_aref_operation(collection, request)
            }
        }
    }
    
    /// Handle composed container operations
    fn handle_composed_operation(
        &self,
        container: &AtomRefContainer,
        element_type: &AtomRefType,
        element_config: &ElementConfig,
        request: &FieldValueSetRequest
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse the request to determine container key and element operation
        let (container_key, element_operation) = self.parse_composed_request(request)?;
        
        // Get or create element ARef
        let element_aref = self.get_or_create_element_aref(
            container,
            &container_key,
            element_type,
            element_config
        )?;
        
        // Apply element operation
        self.apply_element_operation(&element_aref, &element_operation, request)?;
        
        // Update container to reference the element
        self.update_container_element_reference(container, &container_key, &element_aref.uuid())?;
        
        Ok(())
    }
    
    /// Get ARef UUID for field from schema mappings
    fn get_field_aref_uuid(&self, schema_name: &str, field_name: &str) -> Result<String, FieldError> {
        self.schema_manager.get_field_aref_uuid(schema_name, field_name)
            .map_err(|e| FieldError::ARefNotFound(e.to_string()))
    }
    
    /// Retrieve composable ARef
    fn get_composable_aref(&self, aref_uuid: &str) -> Result<ComposableAtomRef, FieldError> {
        self.db_ops.get_composable_aref(aref_uuid)?
            .ok_or_else(|| FieldError::ARefNotFound(aref_uuid.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FieldError {
    #[error("ARef not found: {0}")]
    ARefNotFound(String),
    #[error("Invalid field operation: {0}")]
    InvalidOperation(String),
    #[error("Composable operation failed: {0}")]
    ComposableOperationFailed(String),
}
```

#### Composable Operation Handlers
```rust
impl AtomManager {
    /// Handle Hash AtomRef operations
    fn handle_hash_aref_operation(
        &self,
        hash_aref: &mut AtomRefHash,
        request: &FieldValueSetRequest
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse hash key from request
        let hash_key = self.extract_hash_key(&request.value)?;
        
        // Create atom for the value
        let atom = self.create_atom_for_value(&request.value, &request.schema_name)?;
        
        // Update hash ARef
        hash_aref.set_atom_uuid(hash_key, atom.uuid().to_string());
        
        // Persist the updated ARef
        self.store_composable_aref(&hash_aref.uuid(), 
            &ComposableAtomRef::Single(AtomRefContainer::Hash(hash_aref.clone())))?;
        
        Ok(())
    }
    
    /// Get or create element ARef for composed operations
    fn get_or_create_element_aref(
        &self,
        container: &AtomRefContainer,
        container_key: &str,
        element_type: &AtomRefType,
        element_config: &ElementConfig
    ) -> Result<ComposableAtomRef, Box<dyn std::error::Error>> {
        // Check if element already exists in container
        if let Some(element_uuid) = self.get_element_uuid_from_container(container, container_key)? {
            // Retrieve existing element ARef
            return self.get_composable_aref(&element_uuid);
        }
        
        // Create new element ARef
        let factory = AtomRefFactory::new();
        let element_aref = factory.create_single(
            element_type.clone(),
            element_config.default_source_pub_key.clone()
        )?;
        
        // Store new element ARef
        let element_uuid = element_aref.uuid().to_string();
        self.store_composable_aref(&element_uuid, &element_aref)?;
        
        Ok(element_aref)
    }
    
    /// Parse composed request to extract container key and element operation
    fn parse_composed_request(
        &self,
        request: &FieldValueSetRequest
    ) -> Result<(String, ElementOperation), Box<dyn std::error::Error>> {
        // Request format: { "container_key": "key1", "element_operation": { ... } }
        let request_obj = request.value.as_object()
            .ok_or("Composed request must be an object")?;
        
        let container_key = request_obj.get("container_key")
            .and_then(|v| v.as_str())
            .ok_or("Missing container_key in composed request")?
            .to_string();
        
        let element_operation_value = request_obj.get("element_operation")
            .ok_or("Missing element_operation in composed request")?;
        
        let element_operation = ElementOperation::from_json(element_operation_value)?;
        
        Ok((container_key, element_operation))
    }
}

/// Represents an operation on an element within a composed ARef
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementOperation {
    /// Set a value in the element
    Set { key: Option<String>, value: Value },
    /// Add a value to a collection element
    Add { value: Value },
    /// Remove a value from the element
    Remove { key: Option<String> },
    /// Clear the element
    Clear,
}

impl ElementOperation {
    pub fn from_json(value: &Value) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse JSON into ElementOperation
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid element operation: {}", e).into())
    }
}
```

### Message Bus Integration

#### Composable ARef Events
```rust
/// Events for composable ARef operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComposableARefOperationEvent {
    /// ARef UUID
    pub aref_uuid: String,
    /// Operation type
    pub operation: ComposableOperation,
    /// Source of the operation
    pub source: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComposableOperation {
    /// Single container operation
    SingleContainer {
        container_type: AtomRefType,
        operation: ContainerOperation,
    },
    /// Composed operation (container + element)
    Composed {
        container_type: AtomRefType,
        element_type: AtomRefType,
        container_key: String,
        element_operation: ElementOperation,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerOperation {
    Set { key: Option<String>, value: String },
    Remove { key: Option<String> },
    Clear,
}
```

#### Enhanced Request Handlers
```rust
impl AtomManager {
    /// Handle AtomRef update request for composable types
    pub fn handle_composable_atomref_update(
        &self, 
        request: ComposableARefUpdateRequest
    ) -> Result<(), Box<dyn std::error::Error>> {
        let composable_aref = self.get_composable_aref(&request.aref_uuid)?;
        
        match (&composable_aref, &request.operation) {
            (ComposableAtomRef::Single(container), ComposableOperation::SingleContainer { operation, .. }) => {
                self.apply_container_operation(container, operation)?;
            }
            (ComposableAtomRef::Composed { container, element_type, element_config }, 
             ComposableOperation::Composed { container_key, element_operation, .. }) => {
                self.handle_composed_update(container, element_type, element_config, container_key, element_operation)?;
            }
            _ => {
                return Err("Operation type doesn't match ARef type".into());
            }
        }
        
        // Store updated ARef
        self.store_composable_aref(&request.aref_uuid, &composable_aref)?;
        
        // Publish success event
        let event = ComposableARefOperationEvent {
            aref_uuid: request.aref_uuid,
            operation: request.operation,
            source: "AtomManager".to_string(),
            timestamp: Utc::now(),
        };
        
        self.message_bus.publish(event)?;
        
        Ok(())
    }
}

/// Request for composable ARef updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComposableARefUpdateRequest {
    /// Correlation ID
    pub correlation_id: String,
    /// ARef UUID to update
    pub aref_uuid: String,
    /// Operation to perform
    pub operation: ComposableOperation,
    /// Source public key
    pub source_pub_key: String,
}
```

### HTTP API Updates

#### Composable Field Endpoints
```rust
/// Enhanced API handlers for composable field operations
impl ApiHandlers {
    /// Set field value with composable support
    pub async fn set_field_value_composable(
        State(app_state): State<AppState>,
        Path((schema_name, field_name)): Path<(String, String)>,
        Json(request): Json<ComposableFieldValueRequest>
    ) -> Result<Json<ComposableFieldValueResponse>, ApiError> {
        let field_value_request = FieldValueSetRequest {
            correlation_id: Uuid::new_v4().to_string(),
            schema_name: schema_name.clone(),
            field_name: field_name.clone(),
            value: request.value,
            source_pub_key: request.source_pub_key,
        };
        
        let atom_manager = app_state.atom_manager.lock().unwrap();
        let aref_uuid = atom_manager.handle_field_value_set(field_value_request)
            .map_err(|e| ApiError::ProcessingError(e.to_string()))?;
        
        Ok(Json(ComposableFieldValueResponse {
            success: true,
            aref_uuid: Some(aref_uuid),
            error: None,
        }))
    }
    
    /// Get field value with composable support
    pub async fn get_field_value_composable(
        State(app_state): State<AppState>,
        Path((schema_name, field_name)): Path<(String, String)>,
        Query(params): Query<ComposableFieldQuery>
    ) -> Result<Json<ComposableFieldValueResponse>, ApiError> {
        let atom_manager = app_state.atom_manager.lock().unwrap();
        
        // Get ARef UUID for field
        let aref_uuid = atom_manager.get_field_aref_uuid(&schema_name, &field_name)
            .map_err(|e| ApiError::NotFound(e.to_string()))?;
        
        // Retrieve composable ARef
        let composable_aref = atom_manager.get_composable_aref(&aref_uuid)
            .map_err(|e| ApiError::NotFound(e.to_string()))?;
        
        // Extract value based on query parameters
        let value = self.extract_composable_value(&composable_aref, &params)?;
        
        Ok(Json(ComposableFieldValueResponse {
            success: true,
            value: Some(value),
            aref_uuid: Some(aref_uuid),
            error: None,
        }))
    }
    
    /// Extract value from composable ARef based on query
    fn extract_composable_value(
        &self,
        aref: &ComposableAtomRef,
        query: &ComposableFieldQuery
    ) -> Result<Value, ApiError> {
        match aref {
            ComposableAtomRef::Single(container) => {
                self.extract_container_value(container, &query.container_key)
            }
            ComposableAtomRef::Composed { container, .. } => {
                self.extract_composed_value(container, query)
            }
        }
    }
}

/// Request for composable field operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ComposableFieldValueRequest {
    pub value: Value,
    pub source_pub_key: String,
    /// For composed fields: container key and element operation
    pub container_key: Option<String>,
    pub element_operation: Option<ElementOperation>,
}

/// Response for composable field operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ComposableFieldValueResponse {
    pub success: bool,
    pub value: Option<Value>,
    pub aref_uuid: Option<String>,
    pub error: Option<String>,
}

/// Query parameters for composable field retrieval
#[derive(Debug, Serialize, Deserialize)]
pub struct ComposableFieldQuery {
    /// Container key for accessing specific elements
    pub container_key: Option<String>,
    /// Element key for accessing within composed elements
    pub element_key: Option<String>,
    /// Limit for collection results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}
```

### Implementation Plan

#### Phase 1: Field Processing Updates (Days 1-2)
1. Update field value processing to use pre-created ARefs
2. Implement composable operation handlers
3. Add hash ARef operation support
4. Update error handling for missing ARefs

#### Phase 2: Composable Operation Framework (Days 3-4)
1. Implement composed operation parsing and handling
2. Add element ARef creation and management
3. Create ElementOperation types and processing
4. Add comprehensive validation

#### Phase 3: Message Bus Integration (Days 5-6)
1. Add composable ARef events and request types
2. Update request handlers for composable operations
3. Implement event publishing for composable operations
4. Add event routing and subscription support

#### Phase 4: HTTP API Updates (Days 7)
1. Create composable field endpoints
2. Add request/response types for composable operations
3. Implement query support for composed fields
4. Update API documentation

## UX/UI Considerations

### API Experience
- Intuitive request formats for composable operations
- Clear error messages for invalid operations
- Consistent response formats across all field types

### Performance
- Efficient ARef lookup and caching
- Minimal overhead for simple operations
- Optimized queries for composed field access

## Acceptance Criteria

1. **Field Processing Updates**
   - [ ] Field operations use pre-created ARefs instead of runtime creation
   - [ ] All composable ARef type operations supported
   - [ ] Hash ARef operations implemented
   - [ ] Composed field operations working correctly

2. **Request Handler Updates**
   - [ ] Request handlers support all composable ARef operations
   - [ ] ComposableARefUpdateRequest processing implemented
   - [ ] Error handling for missing or invalid ARefs
   - [ ] Event publishing for successful operations

3. **Message Bus Integration**
   - [ ] Composable ARef events and constructors added
   - [ ] Event routing supports composable operations
   - [ ] Request/response types for all composable operations
   - [ ] Event subscription and monitoring working

4. **HTTP API Support**
   - [ ] Composable field endpoints implemented
   - [ ] Request/response types for composable operations
   - [ ] Query support for accessing composed field elements
   - [ ] API documentation updated

5. **Error Handling**
   - [ ] Graceful handling of missing ARefs
   - [ ] Clear error messages for invalid operations
   - [ ] Validation for composable operation requests
   - [ ] Recovery procedures for failed operations

6. **Backward Compatibility**
   - [ ] All existing single-layer operations continue to work
   - [ ] Legacy API endpoints remain functional
   - [ ] No breaking changes to current workflows
   - [ ] Migration path for existing integrations

## Dependencies

- **Prerequisite**: PBI-2 (Hash AtomRef Type and Composable Framework)
- **Prerequisite**: PBI-3 (Extended Field Type System)
- **Prerequisite**: PBI-4 (Schema Purification and ARef Creation)
- **Internal**: Current field processing, message bus, HTTP API
- **External**: axum for HTTP API, existing request/response types

## Open Questions

1. **Performance Impact**: How do composable operations affect field processing performance?
2. **Caching Strategy**: Should we implement caching for frequently accessed composable ARefs?
3. **API Versioning**: Do we need API versioning for composable field endpoints?
4. **Query Optimization**: How can we optimize queries for deeply nested composed fields?

## Related Tasks

This PBI will generate detailed tasks covering:
- Field processing logic updates for composable ARefs
- Composable operation handlers and validation
- Message bus event types and request handlers
- HTTP API endpoints for composable field operations
- Error handling and backward compatibility testing 