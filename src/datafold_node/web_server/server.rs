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
    println!("UI Server - Handling rejection: {:?}", err);
    
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

/// UI Server for DataFold Node
/// This server provides the web UI for managing the DataFold Node
pub struct WebServer {
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    unix_socket_path: Option<PathBuf>,
    auth_manager: Arc<Mutex<WebAuthManager>>,
}

impl WebServer {
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

        // Static files
        let index = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("src/datafold_node/static/index.html"));

        // Serve static files from the static directory
        let css_files = warp::path("css")
            .and(warp::fs::dir("src/datafold_node/static/css"));
            
        let js_files = warp::path("js")
            .and(warp::fs::dir("src/datafold_node/static/js"));
            
        let components_files = warp::path("components")
            .and(warp::fs::dir("src/datafold_node/static/components"));
            
        let static_files = warp::path("static")
            .and(warp::fs::dir("src/datafold_node/static"));
            
        // Serve app files from the apps directory
        let app_files = warp::path("apps")
            .and(warp::fs::dir("apps"));
            
        // Create minimal API routes for UI functionality only
        let ui_api = self.create_ui_api_routes(Arc::clone(&node));
            
        let routes = ui_api
            .or(index)
            .or(css_files)
            .or(js_files)
            .or(components_files)
            .or(static_files)
            .or(app_files);
            
        // Check if we should use Unix socket
        if let Some(socket_path) = &self.unix_socket_path {
            println!("Starting UI server on Unix socket: {}", socket_path.display());
            return unix_socket::run_unix_socket_server(socket_path, routes).await;
        }

        // Otherwise, use TCP socket
        // Try ports in sequence until we find one that works
        let mut current_port = port;
        let max_port = port + 10; // Try up to 10 ports
        
        while current_port <= max_port {
            let socket_addr = std::net::SocketAddr::from((std::net::Ipv4Addr::new(0, 0, 0, 0), current_port));
            println!("Attempting to start UI server on port {}", current_port);
            
        // Try to bind to the port using a standard TcpListener first
        match std::net::TcpListener::bind(socket_addr) {
            Ok(listener) => {
                // Port is available, close the test listener
                drop(listener);
                
                println!("Successfully bound to port {}", current_port);
                println!("Static files configured");
                println!("UI server running at http://0.0.0.0:{}", current_port);
                
                // Add a rejection handler
                let routes = routes.recover(handle_rejection);
                
                // Start the warp server
                warp::serve(routes)
                    .run(socket_addr)
                    .await;
                        
                println!("UI server stopped");
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

    /// Create minimal API routes needed for UI functionality
    fn create_ui_api_routes(
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
            
        // Combine all routes
        list_schemas
            .or(network_status)
            .or(list_nodes)
            .or(list_apps)
            .or(list_apis)
    }
}
