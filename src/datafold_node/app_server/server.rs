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

        // Determine the port to use. If `port` is 0 let the OS decide,
        // otherwise try to find an available port within a small range.
        let chosen_port = if port == 0 {
            0
        } else {
            find_available_port(port, 10)?
        };

        let socket_addr = std::net::SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), chosen_port));
        let (addr, server) = warp::serve(routes).bind_ephemeral(socket_addr);

        println!("App server running at http://{}", addr);
        server.await;
        println!("App server stopped");
        Ok(())
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

fn find_available_port(start_port: u16, attempts: u16) -> Result<u16, std::io::Error> {
    for offset in 0..=attempts {
        let port = start_port + offset;
        let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), port));
        match std::net::TcpListener::bind(addr) {
            Ok(listener) => {
                drop(listener);
                return Ok(port);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        format!("Could not find an available port in range {}-{}", start_port, start_port + attempts),
    ))
}
