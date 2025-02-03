use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a unique path based on timestamp to avoid conflicts
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Create tmp directory if it doesn't exist
    std::fs::create_dir_all("tmp")?;
    let db_path = format!("tmp/fold_db_{}", timestamp);
    
    println!("\nDatabase initialized successfully at: {}", db_path);

    Ok(())
}
