use fold_node::{datafold_node::DataFoldHttpServer, schema::types::Operation};
use crate::test_data::test_helpers::create_test_node;
use reqwest::Client;
use serde_json::Value;
use std::net::TcpListener;
use tokio::{task::JoinHandle, time::Duration};

async fn start_server() -> (JoinHandle<()>, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let node = create_test_node();
    let bind_address = format!("127.0.0.1:{}", port);
    let server = DataFoldHttpServer::new(node, &bind_address).await.unwrap();
    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    (handle, bind_address)
}

#[tokio::test]
async fn test_list_schemas_route() {
    let (handle, addr) = start_server().await;
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
    let (handle, addr) = start_server().await;
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
    let (handle, addr) = start_server().await;
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
    let (handle, addr) = start_server().await;
    let client = Client::new();
    let resp = client
        .get(format!("http://{}/api/samples/schemas", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schemas: Value = resp.json().await.unwrap();
    let schemas_arr = schemas["data"].as_array().unwrap();
    assert!(schemas_arr.contains(&Value::String("UserProfile".to_string())));
    assert!(schemas_arr.contains(&Value::String("ProductCatalog".to_string())));
    assert!(schemas_arr.contains(&Value::String("TransformBase".to_string())));
    assert!(schemas_arr.contains(&Value::String("TransformSchema".to_string())));
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

    // Verify transform sample schemas
    let resp = client
        .get(format!("http://{}/api/samples/schema/TransformBase", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schema: Value = resp.json().await.unwrap();
    assert_eq!(schema["name"], "TransformBase");

    let resp = client
        .get(format!("http://{}/api/samples/schema/TransformSchema", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schema: Value = resp.json().await.unwrap();
    assert_eq!(schema["name"], "TransformSchema");
    handle.abort();
}

#[tokio::test]
async fn test_network_endpoints() {
    let (handle, addr) = start_server().await;
    let client = Client::new();

    let config = serde_json::json!({
        "listen_address": "/ip4/127.0.0.1/tcp/0",
        "discovery_port": 0,
        "max_connections": 5,
        "connection_timeout_secs": 1,
        "announcement_interval_secs": 1,
        "enable_discovery": false
    });
    let resp = client
        .post(format!("http://{}/api/network/init", addr))
        .json(&config)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert!(body.get("success").is_some());

    let resp = client
        .post(format!("http://{}/api/network/start", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client
        .get(format!("http://{}/api/network/status", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let status: Value = resp.json().await.unwrap();
    assert!(status.get("data").is_some());
    let node_id = status["data"]["node_id"].as_str().unwrap().to_string();

    let resp = client
        .post(format!("http://{}/api/network/connect", addr))
        .json(&serde_json::json!({"node_id": node_id}))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client
        .post(format!("http://{}/api/network/discover", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client
        .get(format!("http://{}/api/network/nodes", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let _ = client
        .post(format!("http://{}/api/network/stop", addr))
        .send()
        .await
        .unwrap();

    handle.abort();
}

#[tokio::test]
async fn test_transform_endpoints() {
    let (handle, addr) = start_server().await;
    let client = Client::new();

    let schema_json = serde_json::json!({
        "name": "transform_schema",
        "fields": {
            "computed": {
                "permission_policy": {
                    "read_policy": { "Distance": 0 },
                    "write_policy": { "Distance": 0 },
                    "explicit_read_policy": null,
                    "explicit_write_policy": null
                },
                "ref_atom_uuid": "calc_uuid",
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": { "None": null },
                    "min_payment": null
                },
                "field_mappers": {},
                "field_type": "Single",
                "transform": "transform calc { logic: { 4 + 5; } }"
            }
        },
        "payment_config": { "base_multiplier": 1.0, "min_payment_threshold": 0 }
    });

    let resp = client
        .post(format!("http://{}/api/schema", addr))
        .json(&schema_json)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "{}", resp.text().await.unwrap());

    let resp = client
        .get(format!("http://{}/api/transforms", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"].as_object().unwrap().contains_key("transform_schema.computed"));

    let resp = client
        .post(format!("http://{}/api/transform/transform_schema.computed/run", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"], serde_json::json!(9.0));

    handle.abort();
}

