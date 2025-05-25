use fold_node::{datafold_node::DataFoldHttpServer, schema::types::Operation};
use fold_node::schema::Schema;
use fold_node::datafold_node::sample_manager::SampleManager;
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
    let handle = tokio::spawn(async move { let _ = server.run().await; });
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Load sample schemas via HTTP
    let client = Client::new();
    let sample_manager = SampleManager::new().await.unwrap();
    let mut names = sample_manager.list_schema_samples();
    names.sort();
    for name in names {
        let value = sample_manager.get_schema_sample(&name).unwrap().clone();
        let schema: Schema = serde_json::from_value(value).unwrap();
        client
            .post(format!("http://{}/api/schema", bind_address))
            .json(&schema)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }

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
    let arr = body["data"].as_array().expect("data array");
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["state"], "Loaded");
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
    assert!(schemas_arr.contains(&Value::String("UserProfileView".to_string())));
    assert!(schemas_arr.contains(&Value::String("BlogPostSummary".to_string())));
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

    // Verify new schema samples with field mappers
    let resp = client
        .get(format!("http://{}/api/samples/schema/UserProfileView", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schema: Value = resp.json().await.unwrap();
    assert_eq!(schema["name"], "UserProfileView");

    let resp = client
        .get(format!("http://{}/api/samples/schema/BlogPostSummary", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let schema: Value = resp.json().await.unwrap();
    assert_eq!(schema["name"], "BlogPostSummary");

    // Query samples
    let resp = client
        .get(format!("http://{}/api/samples/queries", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let queries: Value = resp.json().await.unwrap();
    let q_arr = queries["data"].as_array().unwrap();
    assert!(q_arr.contains(&Value::String("BasicUserQuery".to_string())));

    let resp = client
        .get(format!("http://{}/api/samples/query/BasicUserQuery", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let _query: Value = resp.json().await.unwrap();

    // Mutation samples
    let resp = client
        .get(format!("http://{}/api/samples/mutations", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let muts: Value = resp.json().await.unwrap();
    let m_arr = muts["data"].as_array().unwrap();
    assert!(m_arr.contains(&Value::String("CreateUser".to_string())));

    let resp = client
        .get(format!("http://{}/api/samples/mutation/CreateUser", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let _mutation: Value = resp.json().await.unwrap();
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
                "transform": {
                    "logic": "4 + 5",
                    "inputs": [],
                    "output": "transform_schema.computed"
                }
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

#[tokio::test]
async fn test_sample_transform_visible() {
    let (handle, addr) = start_server().await;
    let client = Client::new();

    let resp = client
        .get(format!("http://{}/api/transforms", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"].as_object().unwrap().contains_key("TransformSchema.result"));

    handle.abort();
}

#[tokio::test]
async fn test_unload_schema_keeps_transforms() {
    let (handle, addr) = start_server().await;
    let client = Client::new();

    let schema_json = serde_json::json!({
        "name": "delete_schema",
        "fields": {
            "calc": {
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
                "transform": {
                    "logic": "1 + 1",
                    "inputs": [],
                    "output": "delete_schema.calc"
                }
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
    assert!(body["data"].as_object().unwrap().contains_key("delete_schema.calc"));

    let resp = client
        .delete(format!("http://{}/api/schema/delete_schema", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client
        .get(format!("http://{}/api/schema/delete_schema", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let resp = client
        .get(format!("http://{}/api/transforms", addr))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"].as_object().unwrap().contains_key("delete_schema.calc"));

    handle.abort();
}

