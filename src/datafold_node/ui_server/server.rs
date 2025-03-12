use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::ui_server::types::with_node;
use crate::datafold_node::ui_server::handlers::*;

pub struct UiServer {
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
}

impl UiServer {
    pub fn new(node: Arc<tokio::sync::Mutex<DataFoldNode>>) -> Self {
        Self { node }
    }

    pub async fn run(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let node = Arc::clone(&self.node);

        // API routes
        let api = self.create_api_routes(node);

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

        let routes = api
            .or(index)
            .or(css_files)
            .or(js_files)
            .or(components_files)
            .or(static_files);

        // Try ports in sequence until we find one that works
        let mut current_port = port;
        let max_port = port + 10; // Try up to 10 ports
        
        while current_port <= max_port {
            let socket_addr = std::net::SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), current_port));
            println!("Attempting to start UI server on port {}", current_port);
            
            // Try to bind to the port using a standard TcpListener first
            match std::net::TcpListener::bind(socket_addr) {
                Ok(listener) => {
                    // Port is available, close the test listener
                    drop(listener);
                    
                    println!("Successfully bound to port {}", current_port);
                    println!("Static files configured");
                    println!("UI server running at http://127.0.0.1:{}", current_port);
                    
                    // Start the warp server
                    warp::serve(routes.clone())
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

    fn create_api_routes(
        &self,
        node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // Schema routes
        let list_schemas = warp::path!("api" / "schemas")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_list_schemas);

        let schema = warp::path!("api" / "schema")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(handle_schema);

        let execute = warp::path!("api" / "execute")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(handle_execute);

        let delete_schema = warp::path!("api" / "schema" / String)
            .and(warp::delete())
            .and(with_node(node.clone()))
            .and_then(handle_delete_schema);

        // Network API routes
        let init_network = warp::path!("api" / "network" / "init")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(handle_init_network);

        let start_network = warp::path!("api" / "network" / "start")
            .and(warp::post())
            .and(with_node(node.clone()))
            .and_then(handle_start_network);

        let stop_network = warp::path!("api" / "network" / "stop")
            .and(warp::post())
            .and(with_node(node.clone()))
            .and_then(handle_stop_network);

        let network_status = warp::path!("api" / "network" / "status")
            .and(warp::get())
            .and(with_node(node.clone()))
            .and_then(handle_network_status);

        let discover_nodes = warp::path!("api" / "network" / "discover")
            .and(warp::post())
            .and(with_node(node.clone()))
            .and_then(handle_discover_nodes);

        let connect_to_node = warp::path!("api" / "network" / "connect")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_node(node.clone()))
            .and_then(handle_connect_to_node);

        let list_nodes = warp::path!("api" / "network" / "nodes")
            .and(warp::get())
            .and(with_node(node))
            .and_then(handle_list_nodes);

        // Combine all routes
        list_schemas
            .or(schema)
            .or(execute)
            .or(delete_schema)
            .or(init_network)
            .or(start_network)
            .or(stop_network)
            .or(network_status)
            .or(discover_nodes)
            .or(connect_to_node)
            .or(list_nodes)
    }
}
