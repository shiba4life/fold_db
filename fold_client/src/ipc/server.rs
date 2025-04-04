//! IPC server for FoldClient
//!
//! This module provides the server-side IPC implementation for the FoldClient.

use crate::auth::AuthManager;
use crate::ipc::{AppRequest, AppResponse, get_app_socket_path};
use crate::node::NodeClient;
use crate::Result;
use crate::FoldClientError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// IPC server for FoldClient
pub struct IpcServer {
    /// Directory where app sockets are created
    app_socket_dir: PathBuf,
    /// Authentication manager
    auth_manager: Arc<AuthManager>,
    /// Node client
    node_client: Arc<NodeClient>,
    /// Active connections
    connections: Arc<Mutex<HashMap<String, mpsc::Sender<AppResponse>>>>,
    /// Server tasks
    tasks: Vec<JoinHandle<()>>,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(
        app_socket_dir: PathBuf,
        auth_manager: Arc<AuthManager>,
        node_client: Arc<NodeClient>,
    ) -> Result<Self> {
        // Create the app socket directory if it doesn't exist
        std::fs::create_dir_all(&app_socket_dir)?;

        Ok(Self {
            app_socket_dir,
            auth_manager,
            node_client,
            connections: Arc::new(Mutex::new(HashMap::new())),
            tasks: Vec::new(),
        })
    }

    /// Start the IPC server
    pub async fn start(&mut self) -> Result<()> {
        // Create a channel for communication between the server and the handler tasks
        let (tx, mut rx) = mpsc::channel::<(AppRequest, mpsc::Sender<AppResponse>)>(100);

        // Start the request handler task
        let auth_manager = self.auth_manager.clone();
        let node_client = self.node_client.clone();
        let connections = self.connections.clone();
        let handler_task = tokio::spawn(async move {
            while let Some((request, response_tx)) = rx.recv().await {
                // Process the request
                let response = Self::process_request(&auth_manager, &node_client, request).await;

                // Send the response
                if let Err(e) = response_tx.send(response).await {
                    log::error!("Failed to send response: {}", e);
                }
            }
        });
        self.tasks.push(handler_task);

        // Start the server for each app
        let apps = {
            let auth_manager = self.auth_manager.clone();
            let apps = auth_manager.list_apps()?;
            apps
        };

        for app in apps {
            self.start_app_server(&app.app_id, tx.clone()).await?;
        }

        Ok(())
    }

    /// Start a server for an app
    pub async fn start_app_server(
        &mut self,
        app_id: &str,
        tx: mpsc::Sender<(AppRequest, mpsc::Sender<AppResponse>)>,
    ) -> Result<()> {
        // Get the socket path for the app
        let socket_path = get_app_socket_path(&self.app_socket_dir, app_id);

        // Remove the socket file if it exists
        let _ = std::fs::remove_file(&socket_path);

        // Create the listener
        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| FoldClientError::Ipc(format!("Failed to bind socket: {}", e)))?;

        // Start the server task
        let app_id = app_id.to_string();
        let connections = self.connections.clone();
        let server_task = tokio::spawn(async move {
            log::info!("Started IPC server for app {}", app_id);

            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        // Create a channel for responses
                        let (response_tx, response_rx) = mpsc::channel::<AppResponse>(100);

                        // Store the response sender
                        {
                            let mut connections = connections.lock().unwrap();
                            connections.insert(app_id.clone(), response_tx.clone());
                        }

                        // Handle the connection
                        let app_id = app_id.clone();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, &app_id, tx, response_rx).await {
                                log::error!("Error handling connection for app {}: {}", app_id, e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Error accepting connection: {}", e);
                        break;
                    }
                }
            }
        });
        self.tasks.push(server_task);

        Ok(())
    }

    /// Handle a connection from an app
    async fn handle_connection(
        mut stream: UnixStream,
        app_id: &str,
        tx: mpsc::Sender<(AppRequest, mpsc::Sender<AppResponse>)>,
        mut response_rx: mpsc::Receiver<AppResponse>,
    ) -> Result<()> {
        log::info!("Handling connection for app {}", app_id);

        // Create a channel for responses specific to this connection
        let (conn_tx, mut conn_rx) = mpsc::channel::<AppResponse>(100);

        // Spawn a task to forward responses from the global channel to this connection
        let app_id_clone = app_id.to_string();
        let conn_tx_clone = conn_tx.clone();
        tokio::spawn(async move {
            while let Some(response) = response_rx.recv().await {
                if let Err(e) = conn_tx_clone.send(response).await {
                    log::error!("Failed to forward response for app {}: {}", app_id_clone, e);
                    break;
                }
            }
        });

        loop {
            // Read the request length
            let request_len = match stream.read_u32().await {
                Ok(len) => len as usize,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        // Client disconnected
                        log::info!("Client disconnected for app {}", app_id);
                        break;
                    }
                    return Err(e.into());
                }
            };

            // Read the request
            let mut request_bytes = vec![0u8; request_len];
            stream.read_exact(&mut request_bytes).await?;

            // Deserialize the request
            let request: AppRequest = serde_json::from_slice(&request_bytes)
                .map_err(|e| FoldClientError::Ipc(format!("Failed to deserialize request: {}", e)))?;

            // Verify that the app ID in the request matches the expected app ID
            if request.app_id != app_id {
                let response = AppResponse::error(
                    &request.request_id,
                    &format!("App ID mismatch: expected {}, got {}", app_id, request.app_id),
                );
                let response_bytes = serde_json::to_vec(&response)
                    .map_err(|e| FoldClientError::Ipc(format!("Failed to serialize response: {}", e)))?;
                stream.write_u32(response_bytes.len() as u32).await?;
                stream.write_all(&response_bytes).await?;
                continue;
            }

            // Forward the request to the handler
            if let Err(e) = tx.send((request, conn_tx.clone())).await {
                log::error!("Failed to forward request: {}", e);
                break;
            }

            // Wait for the response
            let response = match conn_rx.recv().await {
                Some(response) => response,
                None => {
                    log::error!("Response channel closed");
                    break;
                }
            };

            // Serialize the response
            let response_bytes = serde_json::to_vec(&response)
                .map_err(|e| FoldClientError::Ipc(format!("Failed to serialize response: {}", e)))?;

            // Send the response length
            stream.write_u32(response_bytes.len() as u32).await?;

            // Send the response
            stream.write_all(&response_bytes).await?;
        }

        Ok(())
    }

    /// Process a request from an app
    async fn process_request(
        auth_manager: &Arc<AuthManager>,
        node_client: &Arc<NodeClient>,
        request: AppRequest,
    ) -> AppResponse {
        // Verify the app token
        match auth_manager.verify_app_token(&request.app_id, &request.token) {
            Ok(true) => {}
            Ok(false) => {
                return AppResponse::error(&request.request_id, "Invalid app token");
            }
            Err(e) => {
                return AppResponse::error(&request.request_id, &format!("Authentication error: {}", e));
            }
        }

        // Verify the request signature if present
        if let Some(signature) = &request.signature {
            // Create a canonical representation of the request for verification
            let mut request_for_verification = request.clone();
            request_for_verification.signature = None;
            let message = match serde_json::to_string(&request_for_verification) {
                Ok(message) => message,
                Err(e) => {
                    return AppResponse::error(
                        &request.request_id,
                        &format!("Failed to serialize request: {}", e),
                    );
                }
            };

            // Decode the signature
            let signature_bytes = match base64::decode(signature) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return AppResponse::error(
                        &request.request_id,
                        &format!("Invalid signature: {}", e),
                    );
                }
            };

            // Verify the signature
            match auth_manager.verify_signature(&request.app_id, message.as_bytes(), &signature_bytes) {
                Ok(true) => {}
                Ok(false) => {
                    return AppResponse::error(&request.request_id, "Invalid signature");
                }
                Err(e) => {
                    return AppResponse::error(
                        &request.request_id,
                        &format!("Signature verification error: {}", e),
                    );
                }
            }
        }

        // Check if the app has permission to perform the operation
        match auth_manager.check_permission(&request.app_id, &request.operation) {
            Ok(true) => {}
            Ok(false) => {
                return AppResponse::error(
                    &request.request_id,
                    &format!("Permission denied for operation: {}", request.operation),
                );
            }
            Err(e) => {
                return AppResponse::error(
                    &request.request_id,
                    &format!("Permission check error: {}", e),
                );
            }
        }

        // Forward the request to the node
        match node_client.send_request(&request.app_id, &request.operation, request.params.clone()).await {
            Ok(response) => {
                AppResponse::success(&request.request_id, response)
            }
            Err(e) => {
                AppResponse::error(&request.request_id, &format!("Node error: {}", e))
            }
        }
    }

    /// Stop the IPC server
    pub async fn stop(&mut self) -> Result<()> {
        // Abort all tasks
        for task in self.tasks.drain(..) {
            task.abort();
        }

        // Remove all socket files
        for app_id in {
            let auth_manager = self.auth_manager.clone();
            let apps = auth_manager.list_apps()?;
            apps.iter().map(|app| app.app_id.clone()).collect::<Vec<_>>()
        } {
            let socket_path = get_app_socket_path(&self.app_socket_dir, &app_id);
            let _ = std::fs::remove_file(socket_path);
        }

        Ok(())
    }
}
