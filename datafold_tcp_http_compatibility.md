# TCP Server HTTP Compatibility Design (Deprecated)

This document provides detailed information about how the DataFold TCP server will be enhanced to handle HTTP requests while maintaining backward compatibility with existing TCP clients.

## 1. Overview

The current DataFold system uses a custom TCP protocol for communication between clients and the server. To support a modern web UI, we need to enhance this server to also handle HTTP requests. The key challenge is to maintain backward compatibility with existing TCP clients while adding HTTP support.

## 2. Protocol Detection Mechanism

### 2.1 Initial Byte Inspection

The server will use a protocol detection mechanism based on inspecting the first few bytes of each incoming connection:

```rust
async fn handle_connection(mut socket: TcpStream, node: Arc<Mutex<DataFoldNode>>) -> FoldDbResult<()> {
    // Buffer for initial bytes to detect protocol
    let mut buffer = [0u8; 8];
    
    // Peek at the first few bytes without consuming them
    match socket.peek(&mut buffer).await {
        Ok(n) if n >= 4 => {
            // Check if this looks like an HTTP request
            // HTTP methods start with GET, POST, PUT, DELETE, HEAD, OPTIONS, etc.
            if &buffer[0..3] == b"GET" || &buffer[0..4] == b"POST" || 
               &buffer[0..3] == b"PUT" || &buffer[0..6] == b"DELETE" || 
               &buffer[0..4] == b"HEAD" || &buffer[0..7] == b"OPTIONS" {
                return handle_http_request(socket, node).await;
            }
            
            // Otherwise, handle as regular TCP request
            return handle_tcp_request(socket, node).await;
        },
        _ => {
            // Not enough data or error, try regular TCP handling
            return handle_tcp_request(socket, node).await;
        }
    }
}
```

This approach allows the server to determine the protocol without consuming any data from the socket, ensuring that the full request is available to the appropriate handler.

### 2.2 Protocol-Specific Handlers

Once the protocol is detected, the connection is passed to the appropriate handler:

```rust
async fn handle_http_request(mut socket: TcpStream, node: Arc<Mutex<DataFoldNode>>) -> FoldDbResult<()> {
    // Parse HTTP request
    let http_request = parse_http_request(&mut socket).await?;
    
    // Process HTTP request
    let http_response = process_http_request(http_request, node).await?;
    
    // Send HTTP response
    send_http_response(&mut socket, http_response).await?;
    
    Ok(())
}

async fn handle_tcp_request(mut socket: TcpStream, node: Arc<Mutex<DataFoldNode>>) -> FoldDbResult<()> {
    // This is the existing TCP request handling code
    // Read the request length
    let request_len = socket.read_u32().await? as usize;
    
    // Read the request
    let mut request_bytes = vec![0u8; request_len];
    socket.read_exact(&mut request_bytes).await?;
    
    // Process the request
    // ... (existing code)
    
    Ok(())
}
```

## 3. HTTP Request Handling

### 3.1 HTTP Request Parsing

The server will include a basic HTTP parser to handle common HTTP methods and headers:

```rust
async fn parse_http_request(socket: &mut TcpStream) -> FoldDbResult<HttpRequest> {
    // Read the HTTP request line by line until we find the end of headers
    let mut request_lines = Vec::new();
    let mut headers = HashMap::new();
    let mut buffer = String::new();
    let mut reader = BufReader::new(socket);
    
    // Read the request line
    reader.read_line(&mut buffer).await?;
    let request_line = buffer.trim().to_string();
    buffer.clear();
    
    // Parse the request line
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(FoldDbError::Config("Invalid HTTP request line".to_string()));
    }
    
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();
    
    // Read headers
    loop {
        reader.read_line(&mut buffer).await?;
        let line = buffer.trim();
        buffer.clear();
        
        if line.is_empty() {
            // Empty line indicates the end of headers
            break;
        }
        
        // Parse header
        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_string();
            let value = line[idx + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }
    
    // Read body if present
    let mut body = Vec::new();
    if let Some(content_length) = headers.get("Content-Length") {
        if let Ok(length) = content_length.parse::<usize>() {
            let mut body_buffer = vec![0u8; length];
            reader.read_exact(&mut body_buffer).await?;
            body = body_buffer;
        }
    }
    
    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    })
}
```

### 3.2 HTTP Response Generation

The server will generate HTTP responses with appropriate status codes, headers, and content:

```rust
struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

async fn send_http_response(socket: &mut TcpStream, response: HttpResponse) -> FoldDbResult<()> {
    // Generate status line
    let status_text = match response.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    
    let status_line = format!("HTTP/1.1 {} {}\r\n", response.status, status_text);
    
    // Generate headers
    let mut headers_text = String::new();
    for (key, value) in &response.headers {
        headers_text.push_str(&format!("{}: {}\r\n", key, value));
    }
    
    // Add Content-Length header if not present
    if !response.headers.iter().any(|(k, _)| k == "Content-Length") {
        headers_text.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    }
    
    // Add Date header if not present
    if !response.headers.iter().any(|(k, _)| k == "Date") {
        let now = chrono::Utc::now();
        headers_text.push_str(&format!("Date: {}\r\n", now.format("%a, %d %b %Y %H:%M:%S GMT")));
    }
    
    // Add Server header
    headers_text.push_str("Server: DataFold/1.0\r\n");
    
    // End of headers
    headers_text.push_str("\r\n");
    
    // Write status line and headers
    socket.write_all(status_line.as_bytes()).await?;
    socket.write_all(headers_text.as_bytes()).await?;
    
    // Write body
    if !response.body.is_empty() {
        socket.write_all(&response.body).await?;
    }
    
    // Flush the socket
    socket.flush().await?;
    
    Ok(())
}
```

## 4. HTTP API Routing

The server will include a routing mechanism to map HTTP paths to handler functions:

```rust
async fn process_http_request(request: HttpRequest, node: Arc<Mutex<DataFoldNode>>) -> FoldDbResult<HttpResponse> {
    // Extract path and method
    let path = request.path.clone();
    let method = request.method.clone();
    
    // Route the request to the appropriate handler
    match (method.as_str(), path.as_str()) {
        // Static file routes
        ("GET", "/") | ("GET", "/index.html") => serve_static_file("index.html").await,
        ("GET", path) if path.starts_with("/css/") => serve_static_file(&path[1..]).await,
        ("GET", path) if path.starts_with("/js/") => serve_static_file(&path[1..]).await,
        ("GET", path) if path.starts_with("/components/") => serve_static_file(&path[1..]).await,
        
        // API routes
        ("GET", "/api/schemas") => handle_list_schemas(node).await,
        ("GET", "/api/schema") => handle_get_schema(request, node).await,
        ("POST", "/api/schema") => handle_create_schema(request, node).await,
        ("PUT", "/api/schema") => handle_update_schema(request, node).await,
        ("DELETE", "/api/schema") => handle_delete_schema(request, node).await,
        
        ("POST", "/api/query") => handle_query(request, node).await,
        ("POST", "/api/mutation") => handle_mutation(request, node).await,
        
        ("GET", "/api/samples/schemas") => handle_list_schema_samples().await,
        ("GET", "/api/samples/queries") => handle_list_query_samples().await,
        ("GET", "/api/samples/mutations") => handle_list_mutation_samples().await,
        ("GET", path) if path.starts_with("/api/samples/schema/") => {
            let name = &path["/api/samples/schema/".len()..];
            handle_get_schema_sample(name).await
        },
        ("GET", path) if path.starts_with("/api/samples/query/") => {
            let name = &path["/api/samples/query/".len()..];
            handle_get_query_sample(name).await
        },
        ("GET", path) if path.starts_with("/api/samples/mutation/") => {
            let name = &path["/api/samples/mutation/".len()..];
            handle_get_mutation_sample(name).await
        },
        
        // Default: Not Found
        _ => Ok(HttpResponse {
            status: 404,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: b"Not Found".to_vec(),
        }),
    }
}
```

## 5. Maintaining TCP Protocol Compatibility

### 5.1 Protocol Separation

The key to maintaining compatibility is to keep the TCP protocol handling intact while adding HTTP support. The protocol detection mechanism ensures that each connection is handled by the appropriate protocol handler without interference.

### 5.2 Shared Data Access

Both protocol handlers will access the same `DataFoldNode` instance, ensuring that operations performed through either protocol affect the same underlying data:

```rust
// Both handlers use the same node instance
let node_clone = Arc::clone(&self.node);

// HTTP handler
tokio::spawn(async move {
    if let Err(e) = handle_http_request(socket_http, node_clone).await {
        eprintln!("Error handling HTTP connection: {}", e);
    }
});

// TCP handler
let node_clone = Arc::clone(&self.node);
tokio::spawn(async move {
    if let Err(e) = handle_tcp_request(socket_tcp, node_clone).await {
        eprintln!("Error handling TCP connection: {}", e);
    }
});
```

### 5.3 Backward Compatibility Guarantees

To ensure backward compatibility, we will:

1. **Preserve the TCP Protocol Format**: The existing TCP protocol format (length-prefixed JSON messages) will be maintained exactly as is.

2. **Maintain API Compatibility**: All existing TCP API operations will continue to work without changes.

3. **Error Handling**: If protocol detection fails, the server will default to TCP protocol handling.

4. **Connection Handling**: The server will continue to accept connections on the same port, allowing existing clients to connect without changes.

## 6. Error Handling

### 6.1 Protocol Detection Errors

If there's an error during protocol detection, the server will default to TCP protocol handling:

```rust
match socket.peek(&mut buffer).await {
    Ok(n) if n >= 4 => {
        // Protocol detection logic
    },
    _ => {
        // Default to TCP handling for any errors or insufficient data
        return handle_tcp_request(socket, node).await;
    }
}
```

### 6.2 HTTP-Specific Errors

HTTP errors will be returned as proper HTTP error responses:

```rust
fn http_error(status: u16, message: &str) -> FoldDbResult<HttpResponse> {
    Ok(HttpResponse {
        status,
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        body: serde_json::to_vec(&serde_json::json!({
            "error": message
        }))?,
    })
}
```

### 6.3 TCP-Specific Errors

TCP errors will continue to be handled as they are in the existing implementation, with error responses in the TCP protocol format.

## 7. Performance Considerations

### 7.1 Connection Pooling

To handle a large number of connections efficiently, the server will use connection pooling:

```rust
// Create a connection pool with a maximum of 100 connections
let pool = ConnectionPool::new(100);

loop {
    let (socket, _) = self.listener.accept().await?;
    
    // Get a connection from the pool
    let conn = pool.get().await?;
    
    // Clone the node reference for the new connection
    let node_clone = self.node.clone();
    
    // Spawn a new task to handle the connection
    tokio::spawn(async move {
        if let Err(e) = Self::handle_connection(socket, node_clone, conn).await {
            eprintln!("Error handling connection: {}", e);
        }
        
        // Return the connection to the pool
        pool.return_connection(conn).await;
    });
}
```

### 7.2 Protocol Detection Optimization

To optimize protocol detection, we'll use a fast path for known clients:

```rust
async fn handle_connection(mut socket: TcpStream, node: Arc<Mutex<DataFoldNode>>, conn: PoolConnection) -> FoldDbResult<()> {
    // Check if this is a known client
    let peer_addr = socket.peer_addr()?;
    
    // Fast path for known TCP clients
    if KNOWN_TCP_CLIENTS.contains(&peer_addr.ip()) {
        return handle_tcp_request(socket, node, conn).await;
    }
    
    // Fast path for known HTTP clients
    if KNOWN_HTTP_CLIENTS.contains(&peer_addr.ip()) {
        return handle_http_request(socket, node, conn).await;
    }
    
    // Unknown client, perform protocol detection
    // ... (protocol detection logic)
}
```

### 7.3 Asynchronous Processing

Both HTTP and TCP request handling will be fully asynchronous, allowing the server to handle many concurrent connections efficiently:

```rust
// Process multiple requests concurrently
let results = futures::future::join_all(requests.into_iter().map(|req| {
    let node_clone = Arc::clone(&node);
    async move {
        process_request(req, node_clone).await
    }
})).await;
```

## 8. Testing Strategy

To ensure compatibility, we will implement a comprehensive testing strategy:

### 8.1 Protocol Detection Tests

```rust
#[tokio::test]
async fn test_protocol_detection() {
    // Test HTTP detection
    let http_request = b"GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let detected = detect_protocol(&http_request[..]).await;
    assert_eq!(detected, Protocol::Http);
    
    // Test TCP detection
    let tcp_request = [0, 0, 0, 10, /* 10 bytes of data */];
    let detected = detect_protocol(&tcp_request[..]).await;
    assert_eq!(detected, Protocol::Tcp);
}
```

### 8.2 Backward Compatibility Tests

```rust
#[tokio::test]
async fn test_tcp_backward_compatibility() {
    // Create a TCP client using the existing protocol
    let mut client = TcpClient::connect("localhost:9000").await.unwrap();
    
    // Send a request using the existing protocol
    let response = client.send_request(Request::ListSchemas).await.unwrap();
    
    // Verify the response is as expected
    assert!(response.is_ok());
}
```

### 8.3 HTTP Functionality Tests

```rust
#[tokio::test]
async fn test_http_functionality() {
    // Create an HTTP client
    let client = reqwest::Client::new();
    
    // Send an HTTP request
    let response = client.get("http://localhost:9000/api/schemas")
        .send()
        .await
        .unwrap();
    
    // Verify the response is as expected
    assert_eq!(response.status(), 200);
}
```

## 9. Conclusion

By implementing the approach described in this document, we can enhance the existing TCP server to handle HTTP requests while maintaining backward compatibility with existing TCP clients. This will allow us to build a modern web UI for DataFold while preserving compatibility with existing applications.

The key components of this approach are:

1. **Protocol Detection**: Inspecting the first few bytes of each connection to determine the protocol.
2. **Separate Handlers**: Routing connections to the appropriate protocol handler.
3. **Shared Data Access**: Ensuring both protocols operate on the same underlying data.
4. **Backward Compatibility**: Preserving the existing TCP protocol format and API.
5. **Performance Optimization**: Using connection pooling and asynchronous processing.
6. **Comprehensive Testing**: Ensuring both protocols work correctly and maintain compatibility.

This design provides a robust foundation for adding HTTP support to the DataFold TCP server while ensuring existing TCP clients continue to work without modification.