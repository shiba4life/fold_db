use fold_node::schema::core::SchemaCore;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _ = env_logger::init();
    
    println!("🔍 Testing SchemaCore directly...");
    
    // Test 1: Direct file system discovery
    println!("\n=== Test 1: Direct file system discovery ===");
    let available_schemas_path = PathBuf::from("available_schemas");
    
    if available_schemas_path.exists() {
        println!("✅ available_schemas directory exists");
        
        // List files in directory
        match std::fs::read_dir(&available_schemas_path) {
            Ok(entries) => {
                let json_files: Vec<_> = entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.path().extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "json")
                            .unwrap_or(false)
                    })
                    .collect();
                
                println!("📁 Found {} JSON files:", json_files.len());
                for file in &json_files {
                    println!("  - {}", file.file_name().to_string_lossy());
                }
            },
            Err(e) => println!("❌ Failed to read directory: {}", e),
        }
    } else {
        println!("❌ available_schemas directory does not exist");
    }
    
    // Test 2: Create SchemaCore with temp directory (no database issues)
    println!("\n=== Test 2: SchemaCore with temp directory ===");
    let temp_dir = tempfile::tempdir()?;
    let temp_storage_path = temp_dir.path().to_string_lossy().to_string();
    
    match SchemaCore::new(&temp_storage_path) {
        Ok(schema_core) => {
            println!("✅ SchemaCore created successfully with temp storage");
            
            // Test discovery from available_schemas directory
            println!("\n--- Testing discover_available_schemas ---");
            match schema_core.discover_available_schemas() {
                Ok(discovered) => {
                    println!("✅ Schema discovery successful");
                    println!("Discovered {} schemas:", discovered.len());
                    for schema in &discovered {
                        println!("  - {}", schema.name);
                    }
                },
                Err(e) => println!("❌ Schema discovery failed: {}", e),
            }
            
            // Test unified discovery and loading
            println!("\n--- Testing discover_and_load_all_schemas ---");
            match schema_core.discover_and_load_all_schemas() {
                Ok(report) => {
                    println!("✅ Unified discovery and loading successful");
                    println!("Discovered: {:?}", report.discovered_schemas);
                    println!("Loaded: {:?}", report.loaded_schemas);
                    println!("Failed: {:?}", report.failed_schemas);
                },
                Err(e) => println!("❌ Unified discovery and loading failed: {}", e),
            }
            
            // Test list available schemas
            println!("\n--- Testing list_available_schemas ---");
            match schema_core.list_available_schemas() {
                Ok(available) => {
                    println!("✅ list_available_schemas successful");
                    println!("Available {} schemas:", available.len());
                    for schema_name in &available {
                        println!("  - {} (state: {:?})", schema_name, schema_core.get_schema_state(schema_name));
                    }
                },
                Err(e) => println!("❌ list_available_schemas failed: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to create SchemaCore: {}", e),
    }
    
    // Test 3: Try with actual storage path (to see if database is the issue)
    println!("\n=== Test 3: SchemaCore with actual storage path ===");
    let actual_storage_path = "data";
    
    match SchemaCore::new(actual_storage_path) {
        Ok(schema_core) => {
            println!("✅ SchemaCore created with actual storage path");
            
            // Only test discovery (no database operations)
            match schema_core.discover_available_schemas() {
                Ok(discovered) => {
                    println!("✅ Discovery works with actual storage");
                    println!("Discovered {} schemas:", discovered.len());
                    for schema in &discovered {
                        println!("  - {}", schema.name);
                    }
                },
                Err(e) => println!("❌ Discovery failed with actual storage: {}", e),
            }
            
            // Try unified discovery and loading
            match schema_core.discover_and_load_all_schemas() {
                Ok(report) => {
                    println!("✅ Unified discovery works with actual storage");
                    println!("Report: {:?}", report);
                },
                Err(e) => println!("❌ Unified discovery failed with actual storage: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to create SchemaCore with actual storage: {}", e),
    }
    
    println!("\n🏁 Schema core testing complete");
    Ok(())
}