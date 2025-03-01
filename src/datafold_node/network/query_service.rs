use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{FoldDbError, NetworkErrorKind};
use crate::datafold_node::network::connection::Connection;
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::message::{
    Message, QueryMessage, QueryResponseMessage, TrustProof, ErrorCode, ErrorMessage
};
use crate::datafold_node::network::message_router::{MessageHandler, MessageType};
use crate::datafold_node::network::types::{NodeId, QueryResult, SerializableQueryResult};
use crate::schema::types::{Query, SchemaError};

/// Unified service for handling query operations (both client and server functionality)
pub struct QueryService {
    /// Callback for handling query requests
    query_callback: Arc<Mutex<Box<dyn Fn(Query) -> QueryResult + Send + Sync>>>,
    /// Pending queries by ID
    pending_queries: Arc<Mutex<HashMap<Uuid, oneshot::Sender<QueryResult>>>>,
}

impl QueryService {
    /// Creates a new query service
    pub fn new() -> Self {
        // Create default callback
        let query_callback: Box<dyn Fn(Query) -> QueryResult + Send + Sync> = 
            Box::new(|_| Vec::new());
        
        Self {
            query_callback: Arc::new(Mutex::new(query_callback)),
            pending_queries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sets the callback for handling query requests
    pub fn set_query_callback<F>(&mut self, callback: F)
    where
        F: Fn(Query) -> QueryResult + Send + Sync + 'static,
    {
        let mut cb = self.query_callback.lock().unwrap();
        *cb = Box::new(callback);
    }

    /// Executes a query locally
    pub fn execute_query(&self, query: Query) -> QueryResult {
        let callback = self.query_callback.lock().unwrap();
        (*callback)(query)
    }

    /// Validates a trust proof
    fn validate_trust_proof(&self, _trust_proof: &TrustProof) -> NetworkResult<()> {
        // In a real implementation, this would verify the signature and trust distance
        // For now, we'll just accept any trust proof
        Ok(())
    }
    
    /// Sends a query to a remote node
    pub fn query_node(
        &self,
        connection: Arc<Mutex<Connection>>,
        query: Query,
        trust_proof: TrustProof,
    ) -> NetworkResult<QueryResult> {
        // Create query message
        let query_id = Uuid::new_v4();
        let query_message = Message::Query(QueryMessage {
            query_id,
            query,
            trust_proof,
        });
        
        // Create channel for response
        let (tx, mut rx) = oneshot::channel();
        
        // Add to pending queries
        {
            let mut pending = self.pending_queries.lock().unwrap();
            pending.insert(query_id, tx);
        }
        
        // Send query
        {
            let mut connection = connection.lock().unwrap();
            connection.send_message(query_message)?;
        }
        
        // Wait for response with timeout
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        
        // Wait for the response
        while start.elapsed() < timeout {
            // Check if the response has arrived
            if let Ok(result) = rx.try_recv() {
                return Ok(result);
            }
            
            // Sleep briefly before trying again
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Timeout occurred
        Err(FoldDbError::Network(NetworkErrorKind::Timeout("Timeout waiting for query response".to_string())))
    }
}

impl MessageHandler for QueryService {
    fn handle(&self, message: &Message, _node_id: &NodeId) -> NetworkResult<Option<Message>> {
        match message {
            // Server-side handling
            Message::Query(query_msg) => {
                // Validate trust proof
                if let Err(e) = self.validate_trust_proof(&query_msg.trust_proof) {
                    return Ok(Some(Message::Error(ErrorMessage {
                        code: ErrorCode::TrustValidationFailed,
                        message: format!("Trust validation failed: {}", e),
                        details: None,
                        related_message_id: Some(query_msg.query_id),
                    })));
                }
                
                // Execute query
                let result = self.execute_query(query_msg.query.clone());
                
                // Convert QueryResult to SerializableQueryResult
                let serializable_result = SerializableQueryResult::from(result);
                
                // Create response
                let response = Message::QueryResponse(QueryResponseMessage {
                    query_id: query_msg.query_id,
                    result: serializable_result,
                });
                
                Ok(Some(response))
            },
            
            // Client-side handling
            Message::QueryResponse(response) => {
                // Get the pending query
                let sender = {
                    let mut pending = self.pending_queries.lock().unwrap();
                    pending.remove(&response.query_id)
                };
                
                // Send the response
                if let Some(sender) = sender {
                    let _ = sender.send(response.result.clone().into());
                }
                
                Ok(None)
            },
            
            // Error handling
            Message::Error(error) => {
                // Check if this error is related to a pending query
                if let Some(query_id) = error.related_message_id {
                    let sender = {
                        let mut pending = self.pending_queries.lock().unwrap();
                        pending.remove(&query_id)
                    };
                    
                    // Send an error response
                    if let Some(sender) = sender {
                        let error_result = vec![Err(SchemaError::InvalidData(error.message.clone()))];
                        let _ = sender.send(error_result);
                    }
                }
                
                Ok(None)
            },
            
            // Ignore other message types
            _ => Ok(None),
        }
    }

    fn message_types(&self) -> Vec<MessageType> {
        // Handle both query and response messages
        vec![MessageType::Query, MessageType::QueryResponse, MessageType::Error]
    }
}

/// Default implementation
impl Default for QueryService {
    fn default() -> Self {
        Self::new()
    }
}
