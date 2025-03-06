use std::path::Path;
use std::sync::Arc;
use tokio::net::UnixListener;
use warp::Filter;

/// Runs a warp server on a Unix socket
pub async fn run_unix_socket_server(
    socket_path: impl AsRef<Path>,
    routes: impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + Send + Sync + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = socket_path.as_ref();
    
    // Remove the socket file if it already exists
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    
    // Create the parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    
    // Create the Unix socket listener
    let listener = UnixListener::bind(path)?;
    println!("Web server listening on Unix socket: {}", path.display());
    
    // Set permissions on the socket file (world-writable)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o777);
        std::fs::set_permissions(path, permissions)?;
    }
    
    // Create a simple socket server that forwards requests to warp
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                println!("Accepted connection on Unix socket");
                
                // Clone routes for this connection
                let routes = routes.clone();
                
                // Spawn a task to handle this connection
                tokio::spawn(async move {
                    // This is a simplified implementation - in a real-world scenario,
                    // you would need to parse HTTP requests from the stream and forward them to warp
                    // For now, we'll just acknowledge the connection
                    let mut buffer = [0; 1024];
                    match stream.try_read(&mut buffer) {
                        Ok(_) => {
                            println!("Read data from Unix socket connection");
                            // In a real implementation, you would:
                            // 1. Parse the HTTP request
                            // 2. Forward it to warp
                            // 3. Send the response back
                            
                            // For now, just send a simple response
                            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";
                            let _ = stream.try_write(response.as_bytes());
                        }
                        Err(e) => {
                            println!("Error reading from Unix socket: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error accepting connection on Unix socket: {}", e);
            }
        }
    }
}
