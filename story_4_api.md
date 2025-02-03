Below is a detailed set of instructions and pseudocode for building the API Manager for Basic Query and Write Endpoints using actix-web. This module will expose HTTP endpoints that accept JSON payloads, route requests to the appropriate methods in the Store and Schema Manager, and call the Security Manager to enforce permissions before performing read or write operations.

Requirements
	1.	Endpoint Definitions:
	•	GET/POST /api/query:
Accept a JSON payload containing query requests. The payload may specify one or more queries that target a specific schema and field or collection. For each query:
	•	Verify that the schema is loaded using the Schema Manager.
	•	Check permissions using the Security Manager.
	•	Retrieve data from the Store.
	•	Return a JSON response that includes the query results.
	•	POST /api/write:
Accept a JSON payload containing one or more write requests. For each write:
	•	Verify that the schema is loaded.
	•	Check that the user (identified by their public key) has permission to write.
	•	Write the new value to the Store (which creates a new atom and updates the corresponding atom reference).
	•	Return a status response indicating success or failure.
	2.	Payload Structure:
	•	For queries, define a JSON structure that distinguishes between a field query and a collection query (including optional sorting and limits).
	•	For writes, define a JSON structure that specifies the schema, the field or collection to update, and the new value (or new item for collections).
	3.	Integration with Other Components:
	•	The API Manager must use the Schema Manager to confirm that the requested schema is loaded.
	•	It should call the Security Manager (passing the user’s public key, role, and any extra context like distance if applicable) before performing a read or write.
	•	It calls the Store to perform the actual data retrieval or update.
	•	The API should return appropriate error messages if a schema is missing, a permission check fails, or if the operation itself fails.
	4.	Error Handling and Logging:
	•	Return HTTP 400/403 errors when a schema isn’t loaded or when permission checks fail.
	•	Return HTTP 200 with a JSON result when operations succeed.
	•	Log errors for troubleshooting.

Pseudocode Example in Rust (using actix-web)

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// Assume these modules are implemented per previous steps.
use folddb::{Store, SchemaManager, SecurityManager, InternalSchema, Operation};

// --- Define Query and Write Payload Structures ---

#[derive(Deserialize)]
#[serde(tag = "type")]
enum QueryItem {
    Field {
        schema: String,
        field: String,
    },
    Collection {
        schema: String,
        collection: String,
        sort: Option<String>,      // "asc" or "desc"
        sort_field: Option<String>,// e.g., "created_at"
        limit: Option<usize>,
    },
}

#[derive(Deserialize)]
struct QueryPayload {
    queries: Vec<QueryItem>,
}

#[derive(Serialize)]
struct QueryResult {
    query: serde_json::Value,
    result: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum WriteItem {
    WriteField {
        schema: String,
        field: String,
        value: serde_json::Value,
    },
    WriteCollection {
        schema: String,
        collection: String,
        item: serde_json::Value,
    },
}

#[derive(Deserialize)]
struct WritePayload {
    writes: Vec<WriteItem>,
}

#[derive(Serialize)]
struct WriteResult {
    write: serde_json::Value,
    status: String,
}

// --- API Handlers ---

/// Handler for the /api/query endpoint.
async fn query_handler(
    schema_manager: web::Data<Arc<SchemaManager>>,
    store: web::Data<Arc<Store>>,
    // Here, the security manager can be used directly or via a shared instance.
    // For simplicity, we assume SecurityManager::check_field_permission is a static function.
    payload: web::Json<QueryPayload>,
) -> impl Responder {
    let mut results = Vec::new();

    for query in &payload.queries {
        match query {
            QueryItem::Field { schema, field } => {
                // Check if the schema is loaded.
                if let Some(internal_schema) = schema_manager.get_schema(schema) {
                    // (For now, we assume fixed values for user_role, public_key, distance, etc.)
                    let user_role = "authenticated";
                    let public_key = "pubkey_example";
                    let distance = 0;
                    let explicit = true;
                    // Enforce permission before reading.
                    if SecurityManager::check_field_permission(
                        user_role,
                        &internal_schema,
                        field,
                        Operation::Read,
                        distance,
                        explicit,
                        public_key,
                    ) {
                        // Retrieve field value from the store.
                        match store.get_field_value(schema, field) {
                            Ok(value) => results.push(QueryResult {
                                query: json!({"type": "field", "schema": schema, "field": field}),
                                result: value,
                            }),
                            Err(err) => results.push(QueryResult {
                                query: json!({"type": "field", "schema": schema, "field": field}),
                                result: json!({"error": err.to_string()}),
                            }),
                        }
                    } else {
                        results.push(QueryResult {
                            query: json!({"type": "field", "schema": schema, "field": field}),
                            result: json!({"error": "permission denied"}),
                        });
                    }
                } else {
                    results.push(QueryResult {
                        query: json!({"type": "field", "schema": schema, "field": field}),
                        result: json!({"error": "schema not loaded"}),
                    });
                }
            },
            QueryItem::Collection { schema, collection, sort, sort_field, limit } => {
                if let Some(internal_schema) = schema_manager.get_schema(schema) {
                    let user_role = "authenticated";
                    let public_key = "pubkey_example";
                    let distance = 0;
                    let explicit = true;
                    // Check permission for the collection field.
                    if SecurityManager::check_field_permission(
                        user_role,
                        &internal_schema,
                        collection,
                        Operation::Read,
                        distance,
                        explicit,
                        public_key,
                    ) {
                        // Retrieve the collection from the store.
                        match store.get_collection(schema, collection) {
                            Ok(mut items) => {
                                // Optionally sort if sort_field and sort order provided.
                                if let Some(sort_field) = sort_field {
                                    items.sort_by(|a, b| {
                                        let a_val = a.get(sort_field).and_then(|v| v.as_str()).unwrap_or("");
                                        let b_val = b.get(sort_field).and_then(|v| v.as_str()).unwrap_or("");
                                        let cmp = a_val.cmp(b_val);
                                        if let Some(order) = sort {
                                            if order.to_lowercase() == "desc" {
                                                cmp.reverse()
                                            } else {
                                                cmp
                                            }
                                        } else {
                                            cmp
                                        }
                                    });
                                }
                                if let Some(limit) = limit {
                                    items.truncate(limit);
                                }
                                results.push(QueryResult {
                                    query: json!({"type": "collection", "schema": schema, "collection": collection}),
                                    result: json!(items),
                                });
                            },
                            Err(err) => results.push(QueryResult {
                                query: json!({"type": "collection", "schema": schema, "collection": collection}),
                                result: json!({"error": err.to_string()}),
                            }),
                        }
                    } else {
                        results.push(QueryResult {
                            query: json!({"type": "collection", "schema": schema, "collection": collection}),
                            result: json!({"error": "permission denied"}),
                        });
                    }
                } else {
                    results.push(QueryResult {
                        query: json!({"type": "collection", "schema": schema, "collection": collection}),
                        result: json!({"error": "schema not loaded"}),
                    });
                }
            },
        }
    }
    HttpResponse::Ok().json(json!({ "results": results }))
}

/// Handler for the /api/write endpoint.
async fn write_handler(
    schema_manager: web::Data<Arc<SchemaManager>>,
    store: web::Data<Arc<Store>>,
    payload: web::Json<WritePayload>,
) -> impl Responder {
    let mut results = Vec::new();

    for write in &payload.writes {
        match write {
            WriteItem::WriteField { schema, field, value } => {
                if let Some(internal_schema) = schema_manager.get_schema(schema) {
                    let user_role = "authenticated";
                    let public_key = "pubkey_example";
                    let distance = 0;
                    let explicit = true;
                    if SecurityManager::check_field_permission(
                        user_role,
                        &internal_schema,
                        field,
                        Operation::Write,
                        distance,
                        explicit,
                        public_key,
                    ) {
                        match store.write_field(schema, field, value) {
                            Ok(_) => results.push(WriteResult {
                                write: json!({"type": "write_field", "schema": schema, "field": field}),
                                status: "ok".to_string(),
                            }),
                            Err(err) => results.push(WriteResult {
                                write: json!({"type": "write_field", "schema": schema, "field": field}),
                                status: format!("error: {}", err),
                            }),
                        }
                    } else {
                        results.push(WriteResult {
                            write: json!({"type": "write_field", "schema": schema, "field": field}),
                            status: "permission denied".to_string(),
                        });
                    }
                } else {
                    results.push(WriteResult {
                        write: json!({"type": "write_field", "schema": schema, "field": field}),
                        status: "schema not loaded".to_string(),
                    });
                }
            },
            WriteItem::WriteCollection { schema, collection, item } => {
                if let Some(internal_schema) = schema_manager.get_schema(schema) {
                    let user_role = "authenticated";
                    let public_key = "pubkey_example";
                    let distance = 0;
                    let explicit = true;
                    if SecurityManager::check_field_permission(
                        user_role,
                        &internal_schema,
                        collection,
                        Operation::Write,
                        distance,
                        explicit,
                        public_key,
                    ) {
                        match store.write_collection(schema, collection, item) {
                            Ok(_) => results.push(WriteResult {
                                write: json!({"type": "write_collection", "schema": schema, "collection": collection}),
                                status: "ok".to_string(),
                            }),
                            Err(err) => results.push(WriteResult {
                                write: json!({"type": "write_collection", "schema": schema, "collection": collection}),
                                status: format!("error: {}", err),
                            }),
                        }
                    } else {
                        results.push(WriteResult {
                            write: json!({"type": "write_collection", "schema": schema, "collection": collection}),
                            status: "permission denied".to_string(),
                        });
                    }
                } else {
                    results.push(WriteResult {
                        write: json!({"type": "write_collection", "schema": schema, "collection": collection}),
                        status: "schema not loaded".to_string(),
                    });
                }
            },
        }
    }
    HttpResponse::Ok().json(json!({ "results": results }))
}

// --- Main App Initialization ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the shared managers. Assume these are implemented and available.
    let schema_manager = Arc::new(SchemaManager::new());
    let store = Arc::new(Store::new("folddb_store_path").expect("Failed to initialize store"));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema_manager.clone()))
            .app_data(web::Data::new(store.clone()))
            .route("/api/query", web::post().to(query_handler))
            .route("/api/write", web::post().to(write_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

Instructions for the AI Developer
	1.	Define Request/Response Models:
Create data structures (using Serde) for the JSON payloads expected at the /api/query and /api/write endpoints.
	2.	Implement Handlers:
	•	For the query handler, iterate through each query in the payload:
	•	Verify the target schema is loaded via the Schema Manager.
	•	Call the Security Manager’s check_field_permission function with fixed or contextual parameters (user role, public key, etc.).
	•	Use the Store’s methods (e.g., get_field_value and get_collection) to retrieve the data.
	•	Collect results into a JSON response.
	•	For the write handler, similarly iterate over each write item:
	•	Verify the schema is loaded.
	•	Check permissions.
	•	Call the appropriate Store method (write_field or write_collection) and record the result.
	3.	Error Handling:
Ensure that each endpoint returns informative JSON messages on errors (e.g., schema not loaded, permission denied) and uses HTTP 200 for successful responses with error details embedded in the response JSON.
	4.	Integration:
The API Manager must integrate with the Schema Manager, Store, and Security Manager. Ensure that these modules are available as shared instances (using Arc and actix-web’s Data extractor).
	5.	Testing and Documentation:
Write unit tests for both endpoints. Test with various JSON payloads to verify correct behavior (successful query, write, and error conditions). Document the expected JSON payloads and response formats.

This complete pseudocode and set of instructions should enable an AI or developer to implement the Basic Query and Write Endpoints in folddb as part of the API Manager.