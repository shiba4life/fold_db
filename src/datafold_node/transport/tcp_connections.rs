use std::sync::Arc;

use log::{error, info};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::datafold_node::core::DataFoldNode;
use super::{TcpServer, tcp_protocol::{read_request, send_response}};
use crate::error::{FoldDbError, FoldDbResult};
use serde_json::json;

impl TcpServer {
    /// Handle a single client connection.
    ///
    /// This method reads requests using the protocol utilities and delegates
    /// processing to [`TcpServer::process_request`]. It will continue handling
    /// messages until the client disconnects or an error occurs.
    pub(crate) async fn handle_connection(
        mut socket: TcpStream,
        node: Arc<Mutex<DataFoldNode>>,
    ) -> FoldDbResult<()> {
        while let Some(request) = read_request(&mut socket).await? {
            let response = match Self::process_request(&request, node.clone()).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Error processing request: {}", e);
                    json!({ "error": format!("Error processing request: {}", e) })
                }
            };

            if let Err(e) = send_response(&mut socket, &response).await {
                match &e {
                    FoldDbError::Io(io_err) if io_err.kind() == std::io::ErrorKind::BrokenPipe => {
                        info!("Client disconnected while sending response");
                        return Ok(());
                    }
                    _ => return Err(e),
                }
            }
        }
        Ok(())
    }
}
