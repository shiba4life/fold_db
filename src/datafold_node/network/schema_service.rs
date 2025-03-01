use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{FoldDbError, NetworkErrorKind};
use crate::datafold_node::network::connection::Connection;
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::message::{
    Message, ListSchemasRequestMessage, SchemaListResponseMessage
};
use crate::datafold_node::network::message_router::{MessageHandler, MessageType};
use crate::datafold_node::network::types::{NodeId, SchemaInfo};

/// Unified service for handling schema operations (both client and server functionality)
pub struct SchemaService {
    /// Callback for handling schema list requests
    schema_list_callback: Arc<Mutex<Box<dyn Fn() -> Vec<SchemaInfo> + Send + Sync>>>,
    /// Pending schema list requests by ID
    pending_requests: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Vec<SchemaInfo>>>>>,
}

impl SchemaService {
    /// Creates a new schema service
    pub fn new() -> Self {
        // Create default callback
        let schema_list_callback: Box<dyn Fn() -> Vec<SchemaInfo> + Send + Sync> = 
            Box::new(|| Vec::new());
        
        Self {
            schema_list_callback: Arc::new(Mutex::new(schema_list_callback)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sets the callback for handling schema list requests
    pub fn set_schema_list_callback<F>(&mut self, callback: F)
    where
        F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static,
    {
        let mut cb = self.schema_list_callback.lock().unwrap();
        *cb = Box::new(callback);
    }

    /// Gets the list of available schemas locally
    pub fn list_schemas(&self) -> Vec<SchemaInfo> {
        let callback = self.schema_list_callback.lock().unwrap();
        (*callback)()
    }
    
    /// Lists available schemas on a remote node
    pub fn list_remote_schemas(
        &self,
        connection: Arc<Mutex<Connection>>,
    ) -> NetworkResult<Vec<SchemaInfo>> {
        // Create request message
        let request_id = Uuid::new_v4();
        let request_message = Message::ListSchemasRequest(ListSchemasRequestMessage {
            request_id,
        });
        
        // Create channel for response
        let (tx, mut rx) = oneshot::channel();
        
        // Add to pending requests
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(request_id, tx);
        }
        
        // Send request
        {
            let mut connection = connection.lock().unwrap();
            connection.send_message(request_message)?;
        }
        
        // Wait for response with timeout
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        
        // Wait for the response
        while start.elapsed() < timeout {
            // Check if the response has arrived
            if let Ok(schemas) = rx.try_recv() {
                return Ok(schemas);
            }
            
            // Sleep briefly before trying again
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Timeout occurred
        Err(FoldDbError::Network(NetworkErrorKind::Timeout("Timeout waiting for schema list response".to_string())))
    }
}

impl MessageHandler for SchemaService {
    fn handle(&self, message: &Message, _node_id: &NodeId) -> NetworkResult<Option<Message>> {
        match message {
            // Server-side handling
            Message::ListSchemasRequest(request) => {
                // Get schema list
                let schemas = self.list_schemas();
                
                // Create response
                let response = Message::SchemaListResponse(SchemaListResponseMessage {
                    request_id: request.request_id,
                    schemas,
                });
                
                Ok(Some(response))
            },
            
            // Client-side handling
            Message::SchemaListResponse(response) => {
                // Get the pending request
                let sender = {
                    let mut pending = self.pending_requests.lock().unwrap();
                    pending.remove(&response.request_id)
                };
                
                // Send the response
                if let Some(sender) = sender {
                    let _ = sender.send(response.schemas.clone());
                }
                
                Ok(None)
            },
            
            // Error handling
            Message::Error(error) => {
                // Check if this error is related to a pending request
                if let Some(request_id) = error.related_message_id {
                    let sender = {
                        let mut pending = self.pending_requests.lock().unwrap();
                        pending.remove(&request_id)
                    };
                    
                    // Send an empty response
                    if let Some(sender) = sender {
                        let _ = sender.send(Vec::new());
                    }
                }
                
                Ok(None)
            },
            
            // Ignore other message types
            _ => Ok(None),
        }
    }

    fn message_types(&self) -> Vec<MessageType> {
        // Handle both request and response messages
        vec![MessageType::ListSchemasRequest, MessageType::SchemaListResponse, MessageType::Error]
    }
}

/// Default implementation
impl Default for SchemaService {
    fn default() -> Self {
        Self::new()
    }
}
