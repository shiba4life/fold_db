use fold_node::datafold_node::{http_server::DataFoldHttpServer, DataFoldNode, NodeConfig};
use std::net::TcpListener;
use tempfile::tempdir;

/// Verify that server startup does not reload samples.
#[tokio::test]
async fn server_uses_existing_samples() {
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let bind_addr = format!("127.0.0.1:{}", addr.port());

    let server = DataFoldHttpServer::new(node, &bind_addr)
        .await
        .expect("server init");

    let handle = tokio::spawn(async move { server.run().await.unwrap() });

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let client = reqwest::Client::new();
    let url = format!("http://{}/api/samples/schemas", bind_addr);

    let json_value = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to connect to server")
        .error_for_status()
        .expect("Server returned error status")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    let schemas = json_value
        .get("data")
        .expect("missing data field")
        .as_array()
        .expect("data field is not an array");
    assert!(!schemas.is_empty());

    handle.abort();
    let _ = handle.await;
}

/// Verify that logs endpoint returns data
#[tokio::test]
async fn logs_endpoint_returns_lines() {
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let bind_addr = format!("127.0.0.1:{}", addr.port());

    let server = DataFoldHttpServer::new(node, &bind_addr)
        .await
        .expect("server init");

    let handle = tokio::spawn(async move { server.run().await.unwrap() });

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let client = reqwest::Client::new();
    let url = format!("http://{}/api/logs", bind_addr);

    let logs: serde_json::Value = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .expect("request failed")
        .json()
        .await
        .expect("invalid json");

    assert!(logs.as_array().map(|v| !v.is_empty()).unwrap_or(false));

    handle.abort();
    let _ = handle.await;
}
