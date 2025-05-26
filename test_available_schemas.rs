use fold_node::schema::core::SchemaCore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _ = env_logger::init();
    
    println!("Testing available_schemas integration with SchemaCore");
    
    // Create a SchemaCore instance
    let schema_core = SchemaCore::init_default()?;
    
    // Test discovering available schemas
    println!("\n=== Discovering Available Schemas ===");
    let available_schemas = schema_core.discover_available_schemas()?;
    println!("Found {} schemas in available_schemas directory:", available_schemas.len());
    for schema in &available_schemas {
        println!("  - {} (fields: {})", schema.name, schema.fields.len());
        for (field_name, field) in &schema.fields {
            let field_type = match field {
                fold_node::schema::types::field::FieldVariant::Single(_) => "Single",
                fold_node::schema::types::field::FieldVariant::Collection(_) => "Collection",
                fold_node::schema::types::field::FieldVariant::Range(_) => "Range",
            };
            println!("    - {}: {}", field_name, field_type);
        }
    }
    
    // Test loading available schemas into SchemaCore
    println!("\n=== Loading Available Schemas into SchemaCore ===");
    schema_core.load_available_schemas_from_directory()?;
    
    // List all available schemas
    println!("\n=== All Available Schemas in SchemaCore ===");
    let all_schemas = schema_core.list_available_schemas()?;
    println!("Total schemas loaded: {}", all_schemas.len());
    for schema_name in &all_schemas {
        let state = schema_core.get_schema_state(schema_name);
        println!("  - {} (state: {:?})", schema_name, state);
    }
    
    // Test fetching available schemas (combined from both directories)
    println!("\n=== Fetching All Available Schemas ===");
    let fetched_schemas = schema_core.fetch_available_schemas()?;
    println!("Fetched {} schema names:", fetched_schemas.len());
    for schema_name in &fetched_schemas {
        println!("  - {}", schema_name);
    }
    
    // Test approving a schema
    if let Some(first_schema) = all_schemas.first() {
        println!("\n=== Testing Schema Approval ===");
        println!("Approving schema: {}", first_schema);
        schema_core.approve_schema(first_schema)?;
        
        let state_after = schema_core.get_schema_state(first_schema);
        println!("Schema '{}' state after approval: {:?}", first_schema, state_after);
        
        // Check if it can be queried now
        let can_query = schema_core.can_query_schema(first_schema);
        let can_mutate = schema_core.can_mutate_schema(first_schema);
        println!("Can query: {}, Can mutate: {}", can_query, can_mutate);
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}