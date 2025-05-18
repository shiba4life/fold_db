use fold_node::{datafold_node::{DataFoldHttpServer, NodeConfig, DataFoldNode}, schema::types::Operation};
use reqwest::Client;
use serde_json::Value;
use tempfile::TempDir;
use std::net::TcpListener;
use tokio::{task::JoinHandle, time::Duration};

async fn start_server() -> (JoinHandle<()>, String, TempDir) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let temp_dir = tempfile::tempdir().unwrap();
    let config = NodeConfig {
        storage_path: temp_dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let node = DataFoldNode::new(config).unwrap();
    let bind_address = format!("127.0.0.1:{}", port);
    let server = DataFoldHttpServer::new(node, &bind_address).await.unwrap();
    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    (handle, bind_address, temp_dir)
}

#[tokio::test]
async fn test_list_schemas_route() {
    let (handle, addr, _tmp) = start_server().await;
    let client = Client::new();
    let resp = client
        .get(format!("http://{}/api/schemas", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"].as_array().map(|a| !a.is_empty()).unwrap_or(false));
    handle.abort();
}

#[tokio::test]
async fn test_get_schema_route() {
    let (handle, addr, _tmp) = start_server().await;
    let client = Client::new();
    let resp = client
        .get(format!("http://{}/api/schema/UserProfile", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "UserProfile");
    handle.abort();
}

#[tokio::test]
async fn test_execute_route() {
    let (handle, addr, _tmp) = start_server().await;
    let client = Client::new();
    let operation = Operation::Query {
        schema: "UserProfile".to_string(),
        fields: vec!["username".to_string()],
        filter: None,
    };
    let req_body = serde_json::json!({
        "operation": serde_json::to_string(&operation).unwrap()
    });
    let resp = client
        .post(format!("http://{}/api/execute", addr))
        .json(&req_body)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert!(body.get("data").is_some());
    handle.abort();
}

#[tokio::test]
async fn test_sample_endpoints() {
    let (handle, addr, _tmp) = start_server().await;
    let client = Client::new();
    let resp = client
        .get(format!("http://{}/api/samples/schemas", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schemas: Value = resp.json().await.unwrap();
    assert!(schemas.as_array().unwrap().contains(&Value::String("UserProfile".to_string())));
    assert!(schemas.as_array().unwrap().contains(&Value::String("ProductCatalog".to_string())));
    let resp = client
        .get(format!("http://{}/api/samples/schema/UserProfile", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schema: Value = resp.json().await.unwrap();
    assert_eq!(schema["name"], "UserProfile");

    // Verify second sample as well
    let resp = client
        .get(format!("http://{}/api/samples/schema/ProductCatalog", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schema: Value = resp.json().await.unwrap();
    assert_eq!(schema["name"], "ProductCatalog");
    handle.abort();
}

