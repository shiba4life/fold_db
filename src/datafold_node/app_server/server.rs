use std::sync::Arc;
use std::net::SocketAddr;
use warp::{Filter, Rejection, Reply};
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::app_server::types::{with_node, SignedRequest};
use crate::datafold_node::app_server::handlers::*;
use crate::datafold_node::app_server::middleware::create_cors;
use crate::datafold_node::app_server::logging::AppLogger;
use crate::permissions::permission_manager::PermissionManager;

pub struct AppServer {
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    logger: AppLogger,
    permission_manager: PermissionManager,
}

impl AppServer {
    pub fn new(node: Arc<tokio::sync::Mutex<DataFoldNode>>) -> Self {
        // Create logger
        let logger = AppLogger::new("logs/app_server");
        
        // Create permission manager
        let permission_manager = PermissionManager::new();
        
        Self { 
            node,
            logger,
            permission_manager,
        }
    }

    pub async fn run(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let node = Arc::clone(&self.node);
        let logger = self.logger.clone();

        // API routes
        let api = self.create_api_routes(node, logger);

        // Apply CORS
        let routes = api.with(create_cors());

        // Try ports in sequence until we find one that works
        let mut current_port = port;
        let max_port = port + 10; // Try up to 10 ports
        
        while current_port <= max_port {
            let socket_addr = std::net::SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), current_port));
            println!("Attempting to start App server on port {}", current_port);
            
            // Try to bind to the port using a standard TcpListener first
            match std::net::TcpListener::bind(socket_addr) {
                Ok(listener) => {
                    // Port is available, close the test listener
                    drop(listener);
                    
                    println!("Successfully bound to port {}", current_port);
                    println!("App server running at http://127.0.0.1:{}", current_port);
                    
                    // Start the warp server
                    warp::serve(routes.clone())
                        .run(socket_addr)
                        .await;
                    
                    println!("App server stopped");
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
        logger: AppLogger,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // Status route - no authentication required
        let status = warp::path!("api" / "v1" / "status")
            .and(warp::get())
            .and_then(handle_api_status);

        // Simplified operation handler that doesn't use the complex middleware
        let execute = warp::path!("api" / "v1" / "execute")
            .and(warp::post())
            .and(warp::body::json::<SignedRequest>())
            .and(warp::header::<String>("x-public-key"))
            .and(warp::addr::remote())
            .and(with_node(node.clone()))
            .and(warp::any().map(move || logger.clone()))
            .and_then(|request: SignedRequest, public_key: String, addr: Option<SocketAddr>, node, logger| {
                let client_ip = addr.map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
                let request_id = format!("{}-{}", public_key, request.timestamp);
                
                // Call the handler directly
                handle_signed_operation(request, public_key, client_ip, request_id, node, logger)
            });

        // Combine all routes
        status.or(execute)
    }
}
