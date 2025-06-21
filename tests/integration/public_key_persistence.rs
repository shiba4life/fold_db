use datafold::security::{SecurityManager, SecurityConfig, ClientSecurity};
use datafold::db_operations::DbOperations;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_public_key_persistence_across_restart() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path();

    let config = SecurityConfig {
        encrypt_at_rest: false,
        ..Default::default()
    };
    let key_id = {
        let db = sled::open(db_path).unwrap();
        let db_ops = Arc::new(DbOperations::new(db).unwrap());
        let manager = SecurityManager::new_with_persistence(config.clone(), db_ops.clone()).unwrap();

        let keypair = ClientSecurity::generate_client_keypair().unwrap();
        let request = ClientSecurity::create_registration_request(
            &keypair,
            "user1".to_string(),
            vec!["read".to_string()],
        );
        let response = manager.register_public_key(request).unwrap();
        response.public_key_id.unwrap()
        // manager and db_ops dropped here
    };

    // Re-open database and create new manager
    let db = sled::open(db_path).unwrap();
    let db_ops = Arc::new(DbOperations::new(db).unwrap());
    let manager = SecurityManager::new_with_persistence(config, db_ops).unwrap();

    let persisted = manager.get_public_key(&key_id).unwrap();
    assert!(persisted.is_some());
    assert_eq!(persisted.unwrap().owner_id, "user1");
}
