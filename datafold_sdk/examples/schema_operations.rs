use datafold_sdk::{
    DataFoldClient, Schema, SchemaField, FieldType,
    PermissionsPolicy, TrustDistance, FieldPaymentConfig, SchemaPaymentConfig,
    TrustDistanceScaling, AppSdkResult
};
use std::collections::HashMap;
use serde_json::json;

/// Example demonstrating schema operations using the DataFold SDK
///
/// This example shows how to:
/// - Create a schema using the SchemaBuilder
/// - Create a schema manually
/// - Create a schema on the local node
/// - Update a schema
/// - Delete a schema
/// - Get schema details
/// - Discover schemas
///
/// NOTE: This example demonstrates the SDK's schema management API, but
/// requires a DataFold node that supports schema management operations.
/// If you're running this example against a node that doesn't support
/// these operations, you'll see "Unknown operation" errors.
#[tokio::main]
async fn main() -> AppSdkResult<()> {
    // Create a client
    let client = DataFoldClient::new(
        "example_app",
        "private_key_placeholder",
        "public_key_placeholder",
    );

    // Example 1: Create a schema using the SchemaBuilder
    println!("Example 1: Creating a schema using SchemaBuilder");
    let user_profile_schema = client.schema_builder("user_profile")
        .add_field("username", |field| {
            field.field_type(FieldType::Single)
                .required(true)
                .description("User's unique username")
                .permissions(
                    TrustDistance::Distance(0), // Public read
                    TrustDistance::Distance(1)  // Owner-only write
                )
        })
        .add_field("email", |field| {
            field.field_type(FieldType::Single)
                .required(true)
                .description("User's email address")
                .permissions(
                    TrustDistance::Distance(1), // Limited read
                    TrustDistance::Distance(1)  // Owner-only write
                )
                .validation(json!({
                    "format": "email"
                }))
        })
        .add_field("posts", |field| {
            field.field_type(FieldType::Collection)
                .required(false)
                .description("User's posts")
                .permissions(
                    TrustDistance::Distance(0), // Public read
                    TrustDistance::Distance(1)  // Owner-only write
                )
        })
        .payment_config(
            1.0,                           // Base multiplier
            TrustDistanceScaling::None,    // No scaling
            None                           // No threshold
        )
        .build()?;

    // Create the schema on the local node
    // Note: This operation requires server-side support
    match client.create_schema(user_profile_schema).await {
        Ok(_) => println!("User profile schema created successfully"),
        Err(e) => println!("Failed to create schema: {}", e),
    }

    // Example 2: Create a schema manually
    println!("\nExample 2: Creating a schema manually");
    let mut post_fields = HashMap::new();
    
    // Add title field
    post_fields.insert(
        "title".to_string(),
        SchemaField::new(
            PermissionsPolicy {
                read_policy: TrustDistance::Distance(0),  // Public read
                write_policy: TrustDistance::Distance(1), // Owner-only write
            },
            FieldPaymentConfig {
                base_multiplier: 1.0,
                trust_scaling: TrustDistanceScaling::None,
                payment_threshold: None,
            },
            HashMap::new(),
            Some(FieldType::Single),
        ),
    );
    
    // Add content field
    post_fields.insert(
        "content".to_string(),
        SchemaField::new(
            PermissionsPolicy {
                read_policy: TrustDistance::Distance(0),  // Public read
                write_policy: TrustDistance::Distance(1), // Owner-only write
            },
            FieldPaymentConfig {
                base_multiplier: 1.0,
                trust_scaling: TrustDistanceScaling::None,
                payment_threshold: None,
            },
            HashMap::new(),
            Some(FieldType::Single),
        ),
    );
    
    // Add author field
    post_fields.insert(
        "author".to_string(),
        SchemaField::new(
            PermissionsPolicy {
                read_policy: TrustDistance::Distance(0),  // Public read
                write_policy: TrustDistance::Distance(1), // Owner-only write
            },
            FieldPaymentConfig {
                base_multiplier: 1.0,
                trust_scaling: TrustDistanceScaling::None,
                payment_threshold: None,
            },
            HashMap::new(),
            Some(FieldType::Single),
        ),
    );
    
    // Create the post schema
    let post_schema = Schema {
        name: "post".to_string(),
        fields: post_fields,
        payment_config: SchemaPaymentConfig {
            base_multiplier: 1.0,
            trust_scaling: TrustDistanceScaling::None,
            payment_threshold: None,
        },
    };
    
    // Create the schema on the local node
    // Note: This operation requires server-side support
    match client.create_schema(post_schema).await {
        Ok(_) => println!("Post schema created successfully"),
        Err(e) => println!("Failed to create schema: {}", e),
    }

    // Example 3: Get schema details
    println!("\nExample 3: Getting schema details");
    match client.get_schema_details("user_profile", None).await {
        Ok(details) => println!("User profile schema details: {}", details),
        Err(e) => println!("Failed to get schema details: {}", e),
    }

    // Example 4: Discover local schemas
    println!("\nExample 4: Discovering local schemas");
    match client.discover_local_schemas().await {
        Ok(schemas) => println!("Local schemas: {:?}", schemas),
        Err(e) => println!("Failed to discover local schemas: {}", e),
    }

    // Example 5: Update a schema
    println!("\nExample 5: Updating a schema");
    let updated_schema = client.schema_builder("user_profile")
        .add_field("username", |field| {
            field.field_type(FieldType::Single)
                .required(true)
                .description("User's unique username")
        })
        .add_field("email", |field| {
            field.field_type(FieldType::Single)
                .required(true)
                .description("User's email address")
        })
        .add_field("posts", |field| {
            field.field_type(FieldType::Collection)
                .required(false)
                .description("User's posts")
        })
        // Add a new field
        .add_field("bio", |field| {
            field.field_type(FieldType::Single)
                .required(false)
                .description("User's biography")
                .permissions(
                    TrustDistance::Distance(0), // Public read
                    TrustDistance::Distance(1)  // Owner-only write
                )
        })
        .build()?;

    // Update the schema on the local node
    // Note: This operation requires server-side support
    match client.update_schema(updated_schema).await {
        Ok(_) => println!("User profile schema updated successfully"),
        Err(e) => println!("Failed to update schema: {}", e),
    }

    // Example 6: Delete a schema
    println!("\nExample 6: Deleting a schema");
    // Note: This operation requires server-side support
    match client.delete_schema("post").await {
        Ok(_) => println!("Post schema deleted successfully"),
        Err(e) => println!("Failed to delete schema: {}", e),
    }

    // Example 7: Discover schemas on a remote node
    println!("\nExample 7: Discovering schemas on a remote node");
    println!("Note: This example requires a remote node to be available");
    println!("Discovering nodes in the network...");
    
    match client.discover_nodes().await {
        Ok(nodes) => {
            if !nodes.is_empty() {
                let remote_node_id = &nodes[0].id;
                println!("Found node: {}", remote_node_id);
                
                match client.discover_remote_schemas(remote_node_id).await {
                    Ok(schemas) => println!("Schemas on remote node {}: {:?}", remote_node_id, schemas),
                    Err(e) => println!("Failed to discover remote schemas: {}", e),
                }
            } else {
                println!("No remote nodes found");
            }
        },
        Err(e) => println!("Failed to discover nodes: {}", e),
    }

    Ok(())
}
