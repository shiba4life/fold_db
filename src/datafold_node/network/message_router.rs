use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::datafold_node::network::connection::Connection;
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::message::{
    Message, 
    ErrorMessage, ErrorCode
};
use crate::datafold_node::network::types::NodeId;

/// Trait for message handlers
pub trait MessageHandler: Send + Sync {
    /// Handles a message and returns a response message if needed
    fn handle(&self, message: &Message, node_id: &NodeId) -> NetworkResult<Option<Message>>;
    
    /// Returns the message types this handler can handle
    fn message_types(&self) -> Vec<MessageType>;
}

/// Enum representing message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Query message
    Query,
    /// Query response message
    QueryResponse,
    /// List schemas request message
    ListSchemasRequest,
    /// Schema list response message
    SchemaListResponse,
    /// Node announcement message
    NodeAnnouncement,
    /// Error message
    Error,
    /// Ping message
    Ping,
    /// Pong message
    Pong,
}

impl From<&Message> for MessageType {
    fn from(message: &Message) -> Self {
        match message {
            Message::Query(_) => MessageType::Query,
            Message::QueryResponse(_) => MessageType::QueryResponse,
            Message::ListSchemasRequest(_) => MessageType::ListSchemasRequest,
            Message::SchemaListResponse(_) => MessageType::SchemaListResponse,
            Message::NodeAnnouncement(_) => MessageType::NodeAnnouncement,
            Message::Error(_) => MessageType::Error,
            Message::Ping(_) => MessageType::Ping,
            Message::Pong(_) => MessageType::Pong,
        }
    }
}

/// Routes messages to the appropriate handlers
pub struct MessageRouter {
    /// Map of message types to handlers
    handlers: HashMap<MessageType, Vec<Arc<dyn MessageHandler>>>,
}

impl MessageRouter {
    /// Creates a new message router
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers a handler for a message type
    pub fn register_handler(&mut self, handler: Arc<dyn MessageHandler>) {
        for message_type in handler.message_types() {
            self.handlers
                .entry(message_type)
                .or_insert_with(Vec::new)
                .push(Arc::clone(&handler));
        }
    }

    /// Routes a message to the appropriate handlers
    pub fn route_message(&self, message: &Message, node_id: &NodeId) -> NetworkResult<Vec<Message>> {
        let message_type = MessageType::from(message);
        
        // Get handlers for this message type
        let handlers = self.handlers.get(&message_type);
        
        if let Some(handlers) = handlers {
            let mut responses = Vec::new();
            
            // Call each handler
            for handler in handlers {
                if let Some(response) = handler.handle(message, node_id)? {
                    responses.push(response);
                }
            }
            
            Ok(responses)
        } else {
            // No handlers for this message type
            Ok(Vec::new())
        }
    }

    /// Creates an error message
    pub fn create_error(
        code: ErrorCode,
        message: &str,
        details: Option<&str>,
        related_id: Option<Uuid>,
    ) -> Message {
        Message::Error(ErrorMessage {
            code,
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
            related_message_id: related_id,
        })
    }
}

/// Default implementation with empty handlers
impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for processing incoming messages on a connection
/// 
/// Note: This is currently unused but will be used in future implementations
/// for processing messages on connections.
#[allow(dead_code)]
pub struct ConnectionMessageProcessor {
    /// Message router for routing messages
    router: Arc<MessageRouter>,
    /// Connection to process messages for
    connection: Arc<Mutex<Connection>>,
    /// Node ID for the connection
    node_id: NodeId,
}

#[allow(dead_code)]
impl ConnectionMessageProcessor {
    /// Creates a new connection message processor
    pub fn new(
        router: Arc<MessageRouter>,
        connection: Arc<Mutex<Connection>>,
        node_id: NodeId,
    ) -> Self {
        Self {
            router,
            connection,
            node_id,
        }
    }

    /// Processes a single message
    pub fn process_message(&self, message: Message) -> NetworkResult<()> {
        // Route the message to handlers
        let responses = self.router.route_message(&message, &self.node_id)?;
        
        // Send responses
        let mut connection = self.connection.lock().unwrap();
        for response in responses {
            connection.send_message(response)?;
        }
        
        Ok(())
    }

    /// Processes messages in a loop until the connection is closed
    pub fn process_messages(&self) -> NetworkResult<()> {
        while {
            let connection = self.connection.lock().unwrap();
            connection.is_healthy()
        } {
            // Receive message
            let message = {
                let mut connection = self.connection.lock().unwrap();
                match connection.receive_message() {
                    Ok(msg) => msg,
                    Err(e) => {
                        // If the connection is closed, just return
                        if !connection.is_healthy() {
                            return Ok(());
                        }
                        
                        // Otherwise, report the error
                        eprintln!("Error receiving message: {}", e);
                        
                        // Try to send an error message
                        let error = MessageRouter::create_error(
                            ErrorCode::ProtocolError,
                            &format!("Error receiving message: {}", e),
                            None,
                            None,
                        );
                        
                        if let Err(e) = connection.send_message(error) {
                            eprintln!("Error sending error message: {}", e);
                        }
                        
                        // Continue to next message
                        continue;
                    }
                }
            };
            
            // Process the message
            if let Err(e) = self.process_message(message) {
                eprintln!("Error processing message: {}", e);
                
                // Try to send an error message
                let error = MessageRouter::create_error(
                    ErrorCode::InternalError,
                    &format!("Error processing message: {}", e),
                    None,
                    None,
                );
                
                let mut connection = self.connection.lock().unwrap();
                if let Err(e) = connection.send_message(error) {
                    eprintln!("Error sending error message: {}", e);
                }
            }
        }
        
        Ok(())
    }
}
