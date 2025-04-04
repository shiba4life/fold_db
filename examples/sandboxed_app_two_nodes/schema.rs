use datafold_sdk::{
    DataFoldClient,
    types::NodeConnection,
};
use serde_json::json;

use crate::Logger;
use crate::node_setup::TestNode;

/// Create schemas on both nodes
pub async fn create_schemas(_node1: &TestNode, _node2: &TestNode, logger: &mut Logger) -> Result<(), Box<dyn std::error::Error>> {
    logger.log("\nCreating schemas on both nodes...");
    
    // Create clients for both nodes using the TCP server ports
    let client1 = create_client_for_node(1, 8001);
    let client2 = create_client_for_node(2, 8002);
    
    // Create schemas using the client API
    logger.log("Creating schemas using client API");
    
    // Create user schema
    logger.log("Creating user schema");
    let user_schema = get_user_schema();
    
    // Create post schema
    logger.log("Creating post schema");
    let post_schema = get_post_schema();
    
    // Create schemas on Node 1
    logger.log("Creating schemas on Node 1");
    let request1 = client1.send_request(
        datafold_sdk::types::AppRequest::new(
            client1.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": user_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request1 {
        Ok(_) => logger.log("User schema created on Node 1"),
        Err(e) => logger.log(&format!("Error creating user schema on Node 1: {}", e)),
    }
    
    let request1 = client1.send_request(
        datafold_sdk::types::AppRequest::new(
            client1.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": post_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request1 {
        Ok(_) => logger.log("Post schema created on Node 1"),
        Err(e) => logger.log(&format!("Error creating post schema on Node 1: {}", e)),
    }
    
    // Create schemas on Node 2
    logger.log("Creating schemas on Node 2");
    let request2 = client2.send_request(
        datafold_sdk::types::AppRequest::new(
            client2.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": user_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request2 {
        Ok(_) => logger.log("User schema created on Node 2"),
        Err(e) => logger.log(&format!("Error creating user schema on Node 2: {}", e)),
    }
    
    let request2 = client2.send_request(
        datafold_sdk::types::AppRequest::new(
            client2.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": post_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request2 {
        Ok(_) => logger.log("Post schema created on Node 2"),
        Err(e) => logger.log(&format!("Error creating post schema on Node 2: {}", e)),
    }
    
    logger.log("Schemas created on both nodes");
    
    Ok(())
}

/// Create a client for a node
fn create_client_for_node(_node_number: u8, port: u16) -> DataFoldClient {
    let connection = NodeConnection::TcpSocket("127.0.0.1".to_string(), port);
    DataFoldClient::with_connection(
        "schema-creator",
        "private-key-placeholder",
        "public-key-placeholder",
        connection,
    )
}

/// Get the user schema definition
fn get_user_schema() -> serde_json::Value {
    json!({
        "name": "user",
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        },
        "fields": {
            "id": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "username": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "full_name": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "bio": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "email": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            }
        }
    })
}

/// Get the post schema definition
fn get_post_schema() -> serde_json::Value {
    json!({
        "name": "post",
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        },
        "fields": {
            "id": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "title": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "content": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "author_id": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            }
        }
    })
}
