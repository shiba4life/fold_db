//! Request handlers for different types of AtomManager events

use super::AtomManager;
use crate::atom::{Atom, AtomStatus};
use crate::fold_db_core::infrastructure::message_bus::{
    atom_events::{AtomCreated, AtomUpdated, AtomRefCreated, AtomRefUpdated},
    request_events::{
        AtomCreateRequest, AtomCreateResponse, AtomUpdateRequest, AtomUpdateResponse,
        AtomRefCreateRequest, AtomRefCreateResponse, AtomRefUpdateRequest, AtomRefUpdateResponse,
        FieldValueSetRequest,
    },
};
use log::{info, warn};
use std::time::Instant;

impl AtomManager {
    /// Handle AtomCreateRequest by creating atom and publishing response
    pub(super) fn handle_atom_create_request(&self, request: AtomCreateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 Processing AtomCreateRequest for schema: {}", request.schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        let result = self.db_ops.create_atom(
            &request.schema_name,
            request.source_pub_key.clone(),
            request.prev_atom_uuid.clone(),
            request.content.clone(),
            request.status.as_ref().and_then(|s| match s.as_str() {
                "Active" => Some(AtomStatus::Active),
                "Deleted" => Some(AtomStatus::Deleted),
                _ => None,
            }),
        );

        let response = match result {
            Ok(atom) => {
                // Store in memory cache
                self.atoms.lock().unwrap().insert(atom.uuid().to_string(), atom.clone());
                
                // Publish AtomCreated event
                let atom_created = AtomCreated::new(atom.uuid().to_string(), request.content.clone());
                if let Err(e) = self.message_bus.publish(atom_created) {
                    warn!("Failed to publish AtomCreated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atoms_created += 1;
                drop(stats);
                
                AtomCreateResponse::new(
                    request.correlation_id,
                    true,
                    Some(atom.uuid().to_string()),
                    None,
                    Some(request.content),
                )
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomCreateResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(e.to_string()),
                    None,
                )
            }
        };

        // Publish response - Don't fail the operation if response publishing fails
        if let Err(e) = self.message_bus.publish(response) {
            warn!("⚠️ Failed to publish AtomCreateResponse: {}. Operation completed successfully.", e);
        }
        Ok(())
    }

    /// Handle AtomUpdateRequest by updating atom and publishing response
    pub(super) fn handle_atom_update_request(&self, request: AtomUpdateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Processing AtomUpdateRequest for atom: {}", request.atom_uuid);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // For simplicity, we'll create a new atom with the updated content
        // In a real implementation, you might want to update the existing atom
        let atom = Atom::new(
            "default_schema".to_string(),
            request.source_pub_key.clone(),
            request.content.clone(),
        );
        let atom_uuid = atom.uuid().to_string();

        let result = self.db_ops.db().insert(
            format!("atom:{}", atom_uuid),
            serde_json::to_vec(&atom)?,
        );

        let response = match result {
            Ok(_) => {
                // Store in memory cache
                self.atoms.lock().unwrap().insert(atom_uuid.clone(), atom);
                
                // Publish AtomUpdated event
                let atom_updated = AtomUpdated::new(atom_uuid, request.content);
                if let Err(e) = self.message_bus.publish(atom_updated) {
                    warn!("Failed to publish AtomUpdated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atoms_updated += 1;
                drop(stats);
                
                AtomUpdateResponse::new(request.correlation_id, true, None)
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomUpdateResponse::new(request.correlation_id, false, Some(e.to_string()))
            }
        };

        // Publish response - Don't fail the operation if response publishing fails
        if let Err(e) = self.message_bus.publish(response) {
            warn!("⚠️ Failed to publish AtomUpdateResponse: {}. Operation completed successfully.", e);
        }
        Ok(())
    }

    /// Handle AtomRefCreateRequest by creating AtomRef and publishing response
    pub(super) fn handle_atomref_create_request(&self, request: AtomRefCreateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔗 Processing AtomRefCreateRequest for type: {}", request.aref_type);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        let result: Result<(), Box<dyn std::error::Error>> = match request.aref_type.as_str() {
            "Single" => {
                let aref = self.db_ops.update_atom_ref(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_atoms.lock().unwrap().insert(request.aref_uuid.clone(), aref);
                Ok(())
            }
            "Collection" => {
                // TODO: Collections are no longer supported - AtomRefCollection has been removed
                Err("Collection AtomRefs are no longer supported".into())
            }
            "Range" => {
                let range = self.db_ops.update_atom_ref_range(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    "default".to_string(), // Default key
                    request.source_pub_key.clone(),
                )?;
                self.ref_ranges.lock().unwrap().insert(request.aref_uuid.clone(), range);
                Ok(())
            }
            _ => Err(format!("Unknown AtomRef type: {}", request.aref_type).into())
        };

        let response = match result {
            Ok(_) => {
                // Publish AtomRefCreated event
                let atomref_created = AtomRefCreated::new(
                    &request.aref_uuid,
                    &request.aref_type,
                    format!("{}:{}", request.aref_type.to_lowercase(), request.aref_uuid),
                );
                if let Err(e) = self.message_bus.publish(atomref_created) {
                    warn!("Failed to publish AtomRefCreated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atom_refs_created += 1;
                drop(stats);
                
                AtomRefCreateResponse::new(request.correlation_id, true, None)
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomRefCreateResponse::new(request.correlation_id, false, Some(e.to_string()))
            }
        };

        // Publish response - Don't fail the operation if response publishing fails
        if let Err(e) = self.message_bus.publish(response) {
            warn!("⚠️ Failed to publish AtomRefCreateResponse: {}. Operation completed successfully.", e);
        }
        Ok(())
    }

    /// Handle AtomRefUpdateRequest by updating AtomRef and publishing response
    pub(super) fn handle_atomref_update_request(&self, request: AtomRefUpdateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Processing AtomRefUpdateRequest for: {}", request.aref_uuid);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        let result: Result<(), Box<dyn std::error::Error>> = match request.aref_type.as_str() {
            "Single" => {
                let aref = self.db_ops.update_atom_ref(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_atoms.lock().unwrap().insert(request.aref_uuid.clone(), aref);
                Ok(())
            }
            "Collection" => {
                // Handle AtomRefCollection operations
                let action = request.additional_data
                    .as_ref()
                    .and_then(|d| d.get("action"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("add");
                
                match action {
                    "add" => {
                        // Add atom to collection
                        if let Some(collection) = self.ref_collections.lock().unwrap().get_mut(&request.aref_uuid) {
                            collection.add_atom_uuid(request.atom_uuid.clone(), request.source_pub_key.clone());
                            // Store updated collection in database
                            let db_key = format!("ref:{}", request.aref_uuid);
                            self.db_ops.store_item(&db_key, &*collection)?;
                        } else {
                            // Create new collection if it doesn't exist
                            let mut collection = crate::atom::AtomRefCollection::new(request.aref_uuid.clone());
                            collection.add_atom_uuid(request.atom_uuid.clone(), request.source_pub_key.clone());
                            let db_key = format!("ref:{}", request.aref_uuid);
                            self.db_ops.store_item(&db_key, &collection)?;
                            self.ref_collections.lock().unwrap().insert(request.aref_uuid.clone(), collection);
                        }
                        Ok(())
                    }
                    "remove" => {
                        // Remove atom from collection
                        if let Some(collection) = self.ref_collections.lock().unwrap().get_mut(&request.aref_uuid) {
                            collection.remove_atom_uuid(&request.atom_uuid, request.source_pub_key.clone());
                            let db_key = format!("ref:{}", request.aref_uuid);
                            self.db_ops.store_item(&db_key, &*collection)?;
                        }
                        Ok(())
                    }
                    _ => Err(format!("Unknown Collection action: {}", action).into())
                }
            }
            "Range" => {
                let key = request.additional_data
                    .as_ref()
                    .and_then(|d| d.get("key"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let range = self.db_ops.update_atom_ref_range(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    key.to_string(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_ranges.lock().unwrap().insert(request.aref_uuid.clone(), range);
                Ok(())
            }
            _ => Err(format!("Unknown AtomRef type: {}", request.aref_type).into())
        };

        let response = match result {
            Ok(_) => {
                // Publish AtomRefUpdated event
                let atomref_updated = AtomRefUpdated::new(
                    &request.aref_uuid,
                    format!("{}:{}", request.aref_type.to_lowercase(), request.aref_uuid),
                    "update",
                );
                if let Err(e) = self.message_bus.publish(atomref_updated) {
                    warn!("Failed to publish AtomRefUpdated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atom_refs_updated += 1;
                drop(stats);
                
                AtomRefUpdateResponse::new(request.correlation_id, true, None)
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomRefUpdateResponse::new(request.correlation_id, false, Some(e.to_string()))
            }
        };

        // Publish response - Don't fail the operation if response publishing fails
        if let Err(e) = self.message_bus.publish(response) {
            warn!("⚠️ Failed to publish AtomRefUpdateResponse: {}. Operation completed successfully.", e);
        }
        Ok(())
    }

    /// Handle FieldValueSetRequest by creating atom and appropriate AtomRef - CRITICAL MUTATION BUG FIX
    pub(super) fn handle_fieldvalueset_request(&self, request: FieldValueSetRequest) -> Result<(), Box<dyn std::error::Error>> {
        // Delegate to field processing module
        super::field_processing::handle_fieldvalueset_request(self, request)
    }
}