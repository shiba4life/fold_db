use std::sync::Arc;
use std::path::PathBuf;
use warp::{Filter, Rejection, Reply};
use tokio::sync::Mutex;
use serde_json::json;
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::web_server::types::{with_node, ApiErrorResponse};
use crate::datafold_node::web_server::handlers::*;
use crate::datafold_node::web_server::unix_socket;
use crate::datafold_node::web_server::auth::{WebAuthManager, WebAuthConfig, with_auth};

/// Handle rejections and convert them to JSON responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    println!("API Server - Handling rejection: {:?}", err);
    
    // Log the rejection for debugging
    if let Some(e) = err.find::<ApiErrorResponse>() {
        println!("API Error Response: {}", e.error);
        return Ok(warp::reply::json(&json!({
            "error": e.error
        })));
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        println!("Body Deserialize Error: {}", e);
        return Ok(warp::reply::json(&json!({
            "error": format!("Invalid request body: {}", e)
        })));
    } else if let Some(e) = err.find::<warp::reject::MissingHeader>() {
        println!("Missing Header: {}", e);
        return Ok(warp::reply::json(&json!({
            "error": format!("Missing required header: {}", e)
        })));
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        println!("Method Not Allowed");
        return Ok(warp::reply::json(&json!({
            "error": "Method not allowed"
        })));
    } else if let Some(_) = err.find::<warp::reject::InvalidQuery>() {
        println!("Invalid Query");
        return Ok(warp::reply::json(&json!({
            "error": "Invalid query parameters"
        })));
    } else if let Some(_) = err.find::<warp::reject::UnsupportedMediaType>() {
        println!("Unsupported Media Type");
        return Ok(warp::reply::json(&json!({
            "error": "Unsupported media type"
        })));
    } else if let Some(_) = err.find::<warp::reject::PayloadTooLarge>() {
        println!("Payload Too Large");
        return Ok(warp::reply::json(&json!({
            "error": "Payload too large"
        })));
    }
    
    // Fallback for unhandled rejections
    println!("Unhandled rejection: {:?}", err);
    Ok(warp::reply::json(&json!({
        "error": "Unhandled server error"
    })))
}

/// API Server for DataFold Node
/// This server provides API endpoints for other applications to interact with the DataFold Node
pub struct ApiServer {
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    unix_socket_path: Option<PathBuf>,
    auth_manager: Arc<Mutex<WebAuthManager>>,
}

impl ApiServer {
    pub fn new(node: Arc<tokio::sync::Mutex<DataFoldNode>>) -> Self {
        Self { 
            node,
            unix_socket_path: None,
            auth_manager: Arc::new(Mutex::new(WebAuthManager::default())),
        }
    }
    
    pub fn with_auth_config(mut self, config: WebAuthConfig) -> Self {
        self.auth_manager = Arc::new(Mutex::new(WebAuthManager::new(config)));
        self
    }
    
    pub fn with_unix_socket(mut self, socket_path: impl Into<PathBuf>) -> Self {
        self.unix_socket_path = Some(socket_path.into());
        self
    }

    pub async fn run(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let node = Arc::clone(&self.node);

        // API routes
        let api = self.create_api_routes(Arc::clone(&node));
            
        // Check if we should use Unix socket
        if let Some(socket_path) = &self.unix_socket_path {
            println!("Starting API server on Unix socket: {}", socket_path.display());
            return unix_socket::run_unix_socket_server(socket_path, api).await;
        }

        // Otherwise, use TCP socket
        // Try ports in sequence until we find one that works
        let mut current_port = port;
        let max_port = port + 10; // Try up to 10 ports
        
        while current_port <= max_port {
            let socket_addr = std::net::SocketAddr::from((std::net::Ipv4Addr::new(0, 0, 0, 0), current_port));
            println!("Attempting to start API server on port {}", current_port);
            
            // Try to bind to the port using a standard TcpListener first
            match std::net::TcpListener::bind(socket_addr) {
                Ok(listener) => {
                    // Port is available, close the test listener
                    drop(listener);
                    
                    println!("Successfully bound to port {}", current_port);
                    println!("API server running at http://0.0.0.0:{}", current_port);
                    
                    // Add a rejection handler
                    let api = api.recover(handle_rejection);
                    
                    // Start the warp server
                    warp::serve(api)
                        .run(socket_addr)
                        .await;
                            
                    println!("API server stopped");
                    return Ok(());
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::AddrInUse {
                        println!("Port {} is already in use, trying next port...", current_port);
                        current_port += 1;
                    } else {
                        return Err(Box::new(e));
                    }
                }
            }
        }
        
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            format!("Could not find an available port in range {}-{}", port, max_port)
        )))
    }

    fn create_api_routes(
        &self,
        node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // Get auth manager
        let auth_manager = Arc::clone(&self.auth_manager);
        
        // Public routes (no authentication required)
        // Schema routes
        let list_schemas = warp::path!("api" / "schemas")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_list_schemas);
            
        let network_status = warp::path!("api" / "network" / "status")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_network_status);
            
        let list_nodes = warp::path!("api" / "network" / "nodes")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_list_nodes);
            
        let list_apps = warp::path!("api" / "apps")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_list_apps);
            
        let list_apis = warp::path!("api" / "apis")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_list_apis);
            
        // Authenticated routes (require public key authentication)
        // Schema modification routes
        let schema = warp::path!("api" / "schema")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_schema_with_auth(trust_level, body, node)
            });
            
        let load_schema_from_file = warp::path!("api" / "schema" / "load" / "file")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_load_schema_from_file_with_auth(trust_level, body, node)
            });
            
        let load_schema_from_json = warp::path!("api" / "schema" / "load" / "json")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_load_schema_from_json_with_auth(trust_level, body, node)
            });

        let execute = warp::path!("api" / "execute")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_execute_with_auth(trust_level, body, node)
            });

        let delete_schema = warp::path!("api" / "schema" / String)
            .and(warp::delete())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|name, trust_level, node| {
                // Pass trust level to handler
                handle_delete_schema_with_auth(name, trust_level, node)
            });

        // Network API routes
        let init_network = warp::path!("api" / "network" / "init")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_init_network_with_auth(trust_level, body, node)
            });

        let start_network = warp::path!("api" / "network" / "start")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|trust_level, node| {
                // Pass trust level to handler
                handle_start_network_with_auth(trust_level, node)
            });

        let stop_network = warp::path!("api" / "network" / "stop")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|trust_level, node| {
                // Pass trust level to handler
                handle_stop_network_with_auth(trust_level, node)
            });

        let discover_nodes = warp::path!("api" / "network" / "discover")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|trust_level, node| {
                // Pass trust level to handler
                handle_discover_nodes_with_auth(trust_level, node)
            });

        let connect_to_node = warp::path!("api" / "network" / "connect")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_connect_to_node_with_auth(trust_level, body, node)
            });

        // App routes
        let register_app = warp::path!("api" / "apps")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_register_app_with_auth(trust_level, body, node)
            });
            
        let start_app = warp::path!("api" / "apps" / String / "start")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|name, trust_level, node| {
                // Pass trust level to handler
                handle_start_app_with_auth(name, trust_level, node)
            });
            
        let stop_app = warp::path!("api" / "apps" / String / "stop")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|name, trust_level, node| {
                // Pass trust level to handler
                handle_stop_app_with_auth(name, trust_level, node)
            });
            
        let unload_app = warp::path!("api" / "apps" / String)
            .and(warp::delete())
            .and(with_auth(auth_manager.clone()))
            .and(with_node(node.clone()))
            .and_then(|name, trust_level, node| {
                // Pass trust level to handler
                handle_unload_app_with_auth(name, trust_level, node)
            });
            
        let register_api = warp::path!("api" / "apis")
            .and(warp::post())
            .and(with_auth(auth_manager.clone()))
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(|trust_level, body, node| {
                // Pass trust level to handler
                handle_register_api_with_auth(trust_level, body, node)
            });
            
        // Authentication routes
        let register_key = warp::path!("api" / "auth" / "register")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(move |body, node| {
                let auth_manager = Arc::clone(&auth_manager);
                async move {
                    handle_register_key(body, node, auth_manager).await
                }
            });
            
        // Combine all routes
        list_schemas
            .or(schema)
            .or(load_schema_from_file)
            .or(load_schema_from_json)
            .or(execute)
            .or(delete_schema)
            .or(init_network)
            .or(start_network)
            .or(stop_network)
            .or(network_status)
            .or(discover_nodes)
            .or(connect_to_node)
            .or(list_nodes)
            .or(list_apps)
            .or(register_app)
            .or(start_app)
            .or(stop_app)
            .or(unload_app)
            .or(list_apis)
            .or(register_api)
            .or(register_key)
    }
}
