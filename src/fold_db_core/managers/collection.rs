use super::field::FieldManager;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus, AtomRefUpdated,
    FieldValueSetRequest, FieldValueSetResponse,
};
use crate::schema::types::field::common::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::{info, warn};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct CollectionManager {
    pub(super) field_manager: FieldManager,
    message_bus: Arc<MessageBus>,
}

impl CollectionManager {
    pub fn new(field_manager: FieldManager, message_bus: Arc<MessageBus>) -> Self {
        Self {
            field_manager,
            message_bus,
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
