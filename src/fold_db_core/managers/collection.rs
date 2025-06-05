use super::field::FieldManager;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus, AtomRefUpdated, CollectionUpdateRequest, CollectionUpdateResponse,
    FieldValueSetRequest, FieldValueSetResponse,
};
use crate::schema::types::field::common::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::{info, warn, error};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct CollectionManager {
    pub(super) field_manager: FieldManager,
    message_bus: Arc<MessageBus>,
}

impl CollectionManager {
    pub fn new(field_manager: FieldManager, message_bus: Arc<MessageBus>) -> Self {
        let manager = Self {
            field_manager,
            message_bus,
        };
        
        // Start collection update processing
        manager.start_collection_update_processing();
        
        manager
    }
    
    /// Start processing CollectionUpdateRequest events
    fn start_collection_update_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<CollectionUpdateRequest>();
        let manager = self.clone();
        
        tokio::spawn(async move {
            info!("🔄 CollectionManager: Started CollectionUpdateRequest processing");
            
            while let Ok(request) = consumer.recv().await {
                info!("📨 CollectionManager: Processing CollectionUpdateRequest for {}.{}",
                     request.schema_name, request.field_name);
                
                let response = manager.handle_collection_update_request(&request).await;
                
                if let Err(e) = manager.message_bus.publish(response) {
                    error!("❌ Failed to publish CollectionUpdateResponse: {:?}", e);
                }
            }
        })
    }
    
    /// Handle CollectionUpdateRequest for both Collection and Range fields
    async fn handle_collection_update_request(&self, request: &CollectionUpdateRequest) -> CollectionUpdateResponse {
        info!("🔧 Processing collection update for {}.{}", request.schema_name, request.field_name);
        
        // Load schema to determine field type
        let schema = match self.field_manager.schema_manager.get_schema(&request.schema_name) {
            Ok(Some(schema)) => schema,
            Ok(None) => {
                error!("❌ Schema '{}' not found", request.schema_name);
                return CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some(format!("Schema '{}' not found", request.schema_name)),
                    affected_count: 0,
                };
            }
            Err(e) => {
                error!("❌ Error loading schema '{}': {}", request.schema_name, e);
                return CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some(format!("Error loading schema: {}", e)),
                    affected_count: 0,
                };
            }
        };
        
        // Get field definition
        let field_def = match schema.fields.get(&request.field_name) {
            Some(field) => field,
            None => {
                error!("❌ Field '{}' not found in schema '{}'", request.field_name, request.schema_name);
                return CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some(format!("Field '{}' not found", request.field_name)),
                    affected_count: 0,
                };
            }
        };
        
        // Handle based on field type
        match field_def {
            crate::schema::types::field::FieldVariant::Range(_) => {
                info!("🎯 Handling Range field update");
                self.handle_range_field_update(request, &schema).await
            }
            crate::schema::types::field::FieldVariant::Collection(_) => {
                info!("📦 Handling Collection field update");
                self.handle_collection_field_update(request, &schema).await
            }
            crate::schema::types::field::FieldVariant::Single(_) => {
                error!("❌ Single fields should not use CollectionUpdateRequest");
                CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some("Single fields cannot use CollectionUpdateRequest".to_string()),
                    affected_count: 0,
                }
            }
        }
    }
    
    /// Handle Range field updates (the missing piece!)
    async fn handle_range_field_update(&self, request: &CollectionUpdateRequest, schema: &Schema) -> CollectionUpdateResponse {
        info!("🎯 Processing Range field update for {}.{}", schema.name, request.field_name);
        
        // For range fields, we need the item_id as the range key
        let range_key = match &request.item_id {
            Some(key) => key.clone(),
            None => {
                error!("❌ Range field update requires item_id as range key");
                return CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some("Range field update requires item_id".to_string()),
                    affected_count: 0,
                };
            }
        };
        
        info!("🔑 Range key: '{}'", range_key);
        
        // Add range field value using the existing method (adapted for range)
        match self.add_range_field_value(
            schema,
            &request.field_name,
            request.value.clone(),
            request.source_pub_key.clone(),
            range_key,
        ) {
            Ok(_) => {
                info!("✅ Range field value added successfully");
                CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: true,
                    error: None,
                    affected_count: 1,
                }
            }
            Err(e) => {
                error!("❌ Failed to add range field value: {}", e);
                CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some(format!("Failed to add range field value: {}", e)),
                    affected_count: 0,
                }
            }
        }
    }
    
    /// Handle Collection field updates
    async fn handle_collection_field_update(&self, request: &CollectionUpdateRequest, schema: &Schema) -> CollectionUpdateResponse {
        info!("📦 Processing Collection field update for {}.{}", schema.name, request.field_name);
        
        // Use item_id or generate one for collection
        let collection_id = request.item_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        match self.add_collection_field_value(
            schema,
            &request.field_name,
            request.value.clone(),
            request.source_pub_key.clone(),
            collection_id,
        ) {
            Ok(_) => {
                info!("✅ Collection field value added successfully");
                CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: true,
                    error: None,
                    affected_count: 1,
                }
            }
            Err(e) => {
                error!("❌ Failed to add collection field value: {}", e);
                CollectionUpdateResponse {
                    correlation_id: request.correlation_id.clone(),
                    success: false,
                    error: Some(format!("Failed to add collection field value: {}", e)),
                    affected_count: 0,
                }
            }
        }
    }
/// Add a value to a range field (NEW - handles range field updates)
    pub fn add_range_field_value(
        &self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
        range_key: String,
    ) -> Result<(), SchemaError> {
        info!("🎯 Adding range field value for {}.{} with range_key '{}'", schema.name, field, range_key);
        
        // Validate field type
        let field_def = schema.fields.get(field).ok_or_else(|| {
            SchemaError::InvalidData(format!("Field {} not found in schema {}", field, schema.name))
        })?;

        // Check if this is a range field
        if !matches!(field_def, crate::schema::types::field::FieldVariant::Range(_)) {
            return Err(SchemaError::InvalidData(format!(
                "Field {}.{} is not a range field",
                schema.name, field
            )));
        }

        // Check if field has an existing ref_atom_uuid (AtomRefRange)
        let existing_ref_uuid = field_def.ref_atom_uuid().ok_or_else(|| {
            SchemaError::InvalidData(format!(
                "No existing atom_ref found for range field {}.{}. AtomRefRange must be created during field creation.",
                schema.name, field
            ))
        })?;

        info!("🔗 Using existing AtomRefRange: {}", existing_ref_uuid);

        // Create an atom to store the actual data
        let atom = self.field_manager.atom_manager.create_atom(
            &schema.name,
            source_pub_key.clone(),
            None, // no previous version
            content.clone(),
            None, // default status
        )?;

        info!("✅ Created atom {} with content: {}", atom.uuid(), content);

        // Update the AtomRefRange to map range_key -> atom_uuid
        let result = self.field_manager.atom_manager.db_ops.update_atom_ref_range(
            existing_ref_uuid,
            atom.uuid().to_string(),
            range_key.clone(),
            source_pub_key,
        );

        match result {
            Ok(_) => {
                info!("✅ Updated AtomRefRange {} with range_key '{}' -> atom '{}'", 
                     existing_ref_uuid, range_key, atom.uuid());
                
                // Publish AtomRefUpdated event
                let atom_ref_updated = AtomRefUpdated {
                    ref_uuid: existing_ref_uuid.to_string(),
                    field_name: field.to_string(),
                    schema_name: schema.name.clone(),
                    operation: "range_update".to_string(),
                };
                
                if let Err(e) = self.message_bus.publish(atom_ref_updated) {
                    warn!("⚠️ Failed to publish AtomRefUpdated event: {}", e);
                }
                
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update AtomRefRange: {}", e);
                Err(SchemaError::InvalidData(format!("Failed to update AtomRefRange: {}", e)))
            }
        }
    }

    pub fn add_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        // Validate field type
        let field_def = schema.fields.get(field).ok_or_else(|| {
            SchemaError::InvalidData(format!("Field {} not found in schema {}", field, schema.name))
        })?;

        // Check if this is a collection field
        if !matches!(field_def, crate::schema::types::field::FieldVariant::Collection(_)) {
            return Err(SchemaError::InvalidData(format!(
                "Field {}.{} is not a collection field",
                schema.name, field
            )));
        }

        // Check if field has an existing ref_atom_uuid
        let existing_ref_uuid = field_def.ref_atom_uuid().ok_or_else(|| {
            SchemaError::InvalidData(format!(
                "No existing atom_ref found for collection field {}.{}. Atom_refs must be created during field creation.",
                schema.name, field
            ))
        })?;

        // Use the event-driven FieldManager to handle the operation
        let correlation_id = Uuid::new_v4().to_string();
        let request = FieldValueSetRequest {
            correlation_id: correlation_id.clone(),
            schema_name: schema.name.clone(),
            field_name: field.to_string(),
            value: content.clone(),
            source_pub_key: source_pub_key.clone(),
        };

        // Subscribe to responses
        let mut response_consumer = self.message_bus.subscribe::<FieldValueSetResponse>();
        
        // Send the request
        self.message_bus.publish(request)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to send field value set request: {}", e)))?;

        // Wait for the response
        let response = loop {
            if let Ok(response) = response_consumer.recv_timeout(Duration::from_secs(5)) {
                if response.correlation_id == correlation_id {
                    break response;
                }
            } else {
                return Err(SchemaError::InvalidData("Timeout waiting for field value set response".to_string()));
            }
        };

        if !response.success {
            return Err(SchemaError::InvalidData(
                response.error.unwrap_or("Unknown error setting field value".to_string())
            ));
        }

        // Publish additional events for collection-specific tracking
        let field_path = format!("{}.{}", schema.name, field);
        
        // Publish AtomRefUpdated event
        if let Err(e) = self.message_bus.publish(AtomRefUpdated {
            aref_uuid: existing_ref_uuid.to_string(),
            field_path,
            operation: "add".to_string(),
        }) {
            warn!("Failed to publish AtomRefUpdated event: {}", e);
        }

        info!(
            "add_collection_field_value - schema: {}, field: {}, id: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            id,
            existing_ref_uuid
        );

        Ok(())
    }

    pub fn update_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        // Validate field type
        let field_def = schema.fields.get(field).ok_or_else(|| {
            SchemaError::InvalidData(format!("Field {} not found in schema {}", field, schema.name))
        })?;

        if !matches!(field_def, crate::schema::types::field::FieldVariant::Collection(_)) {
            return Err(SchemaError::InvalidData(format!(
                "Field {}.{} is not a collection field",
                schema.name, field
            )));
        }

        // Use the event-driven FieldManager to handle the operation
        let correlation_id = Uuid::new_v4().to_string();
        let request = FieldValueSetRequest {
            correlation_id: correlation_id.clone(),
            schema_name: schema.name.clone(),
            field_name: field.to_string(),
            value: content.clone(),
            source_pub_key: source_pub_key.clone(),
        };

        // Subscribe to responses
        let mut response_consumer = self.message_bus.subscribe::<FieldValueSetResponse>();
        
        // Send the request
        self.message_bus.publish(request)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to send field value set request: {}", e)))?;

        // Wait for the response
        let response = loop {
            if let Ok(response) = response_consumer.recv_timeout(Duration::from_secs(5)) {
                if response.correlation_id == correlation_id {
                    break response;
                }
            } else {
                return Err(SchemaError::InvalidData("Timeout waiting for field value set response".to_string()));
            }
        };

        if !response.success {
            return Err(SchemaError::InvalidData(
                response.error.unwrap_or("Unknown error setting field value".to_string())
            ));
        }

        let aref_uuid = response.aref_uuid.unwrap_or_else(|| {
            field_def.ref_atom_uuid().unwrap_or(&"unknown".to_string()).clone()
        });

        // Publish additional events for collection-specific tracking
        let field_path = format!("{}.{}", schema.name, field);
        
        // Publish AtomRefUpdated event
        if let Err(e) = self.message_bus.publish(AtomRefUpdated {
            aref_uuid: aref_uuid.clone(),
            field_path,
            operation: "update".to_string(),
        }) {
            warn!("Failed to publish AtomRefUpdated event: {}", e);
        }

        info!(
            "update_collection_field_value - schema: {}, field: {}, id: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            id,
            aref_uuid
        );

        Ok(())
    }

    pub fn delete_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        // Validate field type
        let field_def = schema.fields.get(field).ok_or_else(|| {
            SchemaError::InvalidData(format!("Field {} not found in schema {}", field, schema.name))
        })?;

        if !matches!(field_def, crate::schema::types::field::FieldVariant::Collection(_)) {
            return Err(SchemaError::InvalidData(format!(
                "Field {}.{} is not a collection field",
                schema.name, field
            )));
        }

        // Check if field has an existing ref_atom_uuid
        let existing_ref_uuid = field_def.ref_atom_uuid().ok_or_else(|| {
            SchemaError::InvalidData(format!(
                "Cannot delete from collection field {}.{} - no atom_ref exists.",
                schema.name, field
            ))
        })?;

        // Use the event-driven FieldManager to handle the deletion (with null value)
        let correlation_id = Uuid::new_v4().to_string();
        let request = FieldValueSetRequest {
            correlation_id: correlation_id.clone(),
            schema_name: schema.name.clone(),
            field_name: field.to_string(),
            value: Value::Null, // Null indicates deletion
            source_pub_key: source_pub_key.clone(),
        };

        // Subscribe to responses
        let mut response_consumer = self.message_bus.subscribe::<FieldValueSetResponse>();
        
        // Send the request
        self.message_bus.publish(request)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to send field value set request: {}", e)))?;

        // Wait for the response
        let response = loop {
            if let Ok(response) = response_consumer.recv_timeout(Duration::from_secs(5)) {
                if response.correlation_id == correlation_id {
                    break response;
                }
            } else {
                return Err(SchemaError::InvalidData("Timeout waiting for field value set response".to_string()));
            }
        };

        if !response.success {
            return Err(SchemaError::InvalidData(
                response.error.unwrap_or("Unknown error deleting field value".to_string())
            ));
        }

        // Publish additional events for collection-specific tracking
        let field_path = format!("{}.{}", schema.name, field);
        
        // Publish AtomRefUpdated event
        if let Err(e) = self.message_bus.publish(AtomRefUpdated {
            aref_uuid: existing_ref_uuid.to_string(),
            field_path,
            operation: "delete".to_string(),
        }) {
            warn!("Failed to publish AtomRefUpdated event: {}", e);
        }

        info!(
            "delete_collection_field_value - schema: {}, field: {}, id: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            id,
            existing_ref_uuid
        );

        Ok(())
    }
}

impl Clone for CollectionManager {
    fn clone(&self) -> Self {
        Self {
            field_manager: self.field_manager.clone(),
            message_bus: self.message_bus.clone(),
        }
    }
}
