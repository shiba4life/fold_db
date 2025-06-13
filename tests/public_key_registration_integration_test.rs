//! Integration tests for public key registration functionality
//!
//! This module tests the core database operations and validation logic
//! for public key registration without relying on the HTTP layer.

use std::collections::HashMap;
use tempfile::TempDir;

use datafold::crypto::ed25519::generate_master_keypair;
use datafold::datafold_node::{
    crypto_routes::{PublicKeyRegistration, CLIENT_KEY_INDEX_TREE, PUBLIC_KEY_REGISTRATIONS_TREE},
    DataFoldNode, NodeConfig,
};

/// Test fixture for public key registration tests
struct PublicKeyRegistrationTestFixture {
    _temp_dir: TempDir,
    node: DataFoldNode,
}

impl PublicKeyRegistrationTestFixture {
    /// Create a new test fixture with a fresh database
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).expect("Failed to create test node");

        Self {
            _temp_dir: temp_dir,
            node,
        }
    }

    /// Get database operations from the node
    fn db_ops(&self) -> std::sync::Arc<datafold::db_operations::core::DbOperations> {
        let fold_db = self.node.get_fold_db().expect("FoldDB should be available");
        fold_db.db_ops()
    }

    /// Generate a valid Ed25519 public key for testing
    fn generate_test_public_key() -> [u8; 32] {
        let keypair = generate_master_keypair().expect("Failed to generate test keypair");
        keypair.public_key_bytes()
    }

    /// Store a test registration directly in the database
    fn store_test_registration(
        &self,
        registration_id: &str,
        client_id: &str,
        public_key_bytes: [u8; 32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db_ops = self.db_ops();

        let registration = PublicKeyRegistration {
            registration_id: registration_id.to_string(),
            client_id: client_id.to_string(),
            user_id: Some("test-user".to_string()),
            public_key_bytes,
            key_name: Some("Test Key".to_string()),
            metadata: HashMap::new(),
            registered_at: chrono::Utc::now(),
            status: "active".to_string(),
            last_used: None,
        };

        // Store registration
        let registration_key = format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, registration_id);
        db_ops.store_item(&registration_key, &registration)?;

        // Store client index
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, client_id);
        db_ops.store_item(&client_index_key, &registration_id.to_string())?;

        Ok(())
    }
}

#[tokio::test]
async fn test_store_and_retrieve_public_key_registration() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let db_ops = fixture.db_ops();

    let public_key_bytes = PublicKeyRegistrationTestFixture::generate_test_public_key();
    let registration_id = "test-reg-123";
    let client_id = "test-client-1";

    // Store registration
    fixture
        .store_test_registration(registration_id, client_id, public_key_bytes)
        .expect("Failed to store test registration");

    // Retrieve by registration ID
    let registration_key = format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, registration_id);
    let retrieved_registration: PublicKeyRegistration = db_ops
        .get_item(&registration_key)
        .expect("Failed to get registration")
        .expect("Registration should exist");

    assert_eq!(retrieved_registration.registration_id, registration_id);
    assert_eq!(retrieved_registration.client_id, client_id);
    assert_eq!(retrieved_registration.public_key_bytes, public_key_bytes);
    assert_eq!(retrieved_registration.status, "active");

    // Retrieve by client ID
    let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, client_id);
    let retrieved_reg_id: String = db_ops
        .get_item(&client_index_key)
        .expect("Failed to get client index")
        .expect("Client index should exist");

    assert_eq!(retrieved_reg_id, registration_id);
}

#[tokio::test]
async fn test_public_key_validation() {
    use datafold::crypto::PublicKey;

    // Test valid public key
    let valid_key_bytes = PublicKeyRegistrationTestFixture::generate_test_public_key();
    let public_key_result = PublicKey::from_bytes(&valid_key_bytes);
    assert!(
        public_key_result.is_ok(),
        "Valid public key should parse successfully"
    );

    // Test invalid public key (all zeros)
    let invalid_key_bytes = [0u8; 32];
    let invalid_public_key_result = PublicKey::from_bytes(&invalid_key_bytes);
    assert!(
        invalid_public_key_result.is_err(),
        "Invalid public key should fail to parse"
    );
}

#[tokio::test]
async fn test_hex_encoding_decoding() {
    let original_key_bytes = PublicKeyRegistrationTestFixture::generate_test_public_key();

    // Encode to hex
    let hex_string = hex::encode(original_key_bytes);
    assert_eq!(hex_string.len(), 64); // 32 bytes * 2 hex chars = 64 chars

    // Decode from hex
    let decoded_bytes = hex::decode(&hex_string).expect("Hex decoding should succeed");
    assert_eq!(decoded_bytes.len(), 32);

    let mut decoded_array = [0u8; 32];
    decoded_array.copy_from_slice(&decoded_bytes);

    assert_eq!(original_key_bytes, decoded_array);
}

#[tokio::test]
async fn test_duplicate_detection() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let db_ops = fixture.db_ops();

    let public_key_bytes = PublicKeyRegistrationTestFixture::generate_test_public_key();

    // Store first registration
    fixture
        .store_test_registration("reg-1", "client-1", public_key_bytes)
        .expect("First registration should succeed");

    // Check for duplicate by scanning registrations
    let tree_prefix = format!("{}:", PUBLIC_KEY_REGISTRATIONS_TREE);
    let mut found_duplicate = false;

    for item in db_ops.db().scan_prefix(tree_prefix.as_bytes()) {
        if let Ok((_, value)) = item {
            if let Ok(registration) = serde_json::from_slice::<PublicKeyRegistration>(&value) {
                if registration.public_key_bytes == public_key_bytes {
                    found_duplicate = true;
                    break;
                }
            }
        }
    }

    assert!(found_duplicate, "Should find the stored public key");
}

#[tokio::test]
async fn test_client_id_generation() {
    use uuid::Uuid;

    // Test UUID v4 generation for client IDs
    let client_id = format!("client_{}", Uuid::new_v4());
    assert!(client_id.starts_with("client_"));
    assert!(client_id.len() > 7); // "client_" + UUID length
}

#[tokio::test]
async fn test_registration_serialization() {
    let registration = PublicKeyRegistration {
        registration_id: "test-123".to_string(),
        client_id: "client-456".to_string(),
        user_id: Some("user-789".to_string()),
        public_key_bytes: [1u8; 32],
        key_name: Some("Test Key".to_string()),
        metadata: HashMap::from([
            ("env".to_string(), "test".to_string()),
            ("purpose".to_string(), "integration".to_string()),
        ]),
        registered_at: chrono::Utc::now(),
        status: "active".to_string(),
        last_used: None,
    };

    // Test serialization
    let serialized = serde_json::to_vec(&registration).expect("Serialization should succeed");
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized: PublicKeyRegistration =
        serde_json::from_slice(&serialized).expect("Deserialization should succeed");

    assert_eq!(deserialized.registration_id, registration.registration_id);
    assert_eq!(deserialized.client_id, registration.client_id);
    assert_eq!(deserialized.public_key_bytes, registration.public_key_bytes);
    assert_eq!(deserialized.status, registration.status);
}

#[tokio::test]
async fn test_database_operations() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let db_ops = fixture.db_ops();

    // Test basic key-value operations
    let test_key = "test_key";
    let test_value = "test_value";

    db_ops
        .store_item(test_key, &test_value.to_string())
        .expect("Store operation should succeed");

    let retrieved_value: String = db_ops
        .get_item(test_key)
        .expect("Get operation should succeed")
        .expect("Value should exist");

    assert_eq!(retrieved_value, test_value);
}

#[tokio::test]
async fn test_multiple_registrations() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;

    // Store multiple registrations
    let key1 = PublicKeyRegistrationTestFixture::generate_test_public_key();
    let key2 = PublicKeyRegistrationTestFixture::generate_test_public_key();
    let key3 = PublicKeyRegistrationTestFixture::generate_test_public_key();

    fixture
        .store_test_registration("reg-1", "client-1", key1)
        .expect("Registration 1 should succeed");
    fixture
        .store_test_registration("reg-2", "client-2", key2)
        .expect("Registration 2 should succeed");
    fixture
        .store_test_registration("reg-3", "client-3", key3)
        .expect("Registration 3 should succeed");

    // Verify all registrations exist
    let db_ops = fixture.db_ops();

    let reg1: PublicKeyRegistration = db_ops
        .get_item(&format!("{}:reg-1", PUBLIC_KEY_REGISTRATIONS_TREE))
        .expect("Get reg1 should succeed")
        .expect("Reg1 should exist");
    let reg2: PublicKeyRegistration = db_ops
        .get_item(&format!("{}:reg-2", PUBLIC_KEY_REGISTRATIONS_TREE))
        .expect("Get reg2 should succeed")
        .expect("Reg2 should exist");
    let reg3: PublicKeyRegistration = db_ops
        .get_item(&format!("{}:reg-3", PUBLIC_KEY_REGISTRATIONS_TREE))
        .expect("Get reg3 should succeed")
        .expect("Reg3 should exist");

    assert_eq!(reg1.client_id, "client-1");
    assert_eq!(reg2.client_id, "client-2");
    assert_eq!(reg3.client_id, "client-3");

    assert_eq!(reg1.public_key_bytes, key1);
    assert_eq!(reg2.public_key_bytes, key2);
    assert_eq!(reg3.public_key_bytes, key3);
}
