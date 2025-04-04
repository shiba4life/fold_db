//! Authentication module for FoldClient
//!
//! This module handles app registration, token management, and permission checking.

use crate::Result;
use crate::FoldClientError;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand_07::rngs::OsRng as OsRng07;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// App registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistration {
    /// Unique identifier for the app
    pub app_id: String,
    /// Name of the app
    pub app_name: String,
    /// Token for the app
    pub token: String,
    /// Permissions granted to the app
    pub permissions: Vec<String>,
    /// Public key for the app
    pub public_key: String,
    /// When the app was registered
    pub created_at: DateTime<Utc>,
}

/// Authentication manager for FoldClient
pub struct AuthManager {
    /// Directory where app registrations are stored
    app_dir: PathBuf,
    /// Registered apps
    apps: Arc<Mutex<HashMap<String, AppRegistration>>>,
    /// Keypair for the FoldClient
    keypair: Keypair,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        // Create the app directory if it doesn't exist
        fs::create_dir_all(&app_dir)?;

        // Load or generate a keypair
        let keypair_path = app_dir.join("fold_client.key");
        let keypair = if keypair_path.exists() {
            // Load the keypair from file
            let mut file = File::open(&keypair_path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Keypair::from_bytes(&bytes)
                .map_err(|e| FoldClientError::Auth(format!("Failed to load keypair: {}", e)))?
        } else {
            // Generate a new keypair
            let mut csprng = OsRng07{};
            let keypair = Keypair::generate(&mut csprng);

            // Save the keypair to file
            let mut file = File::create(&keypair_path)?;
            file.write_all(&keypair.to_bytes())?;

            keypair
        };

        // Load registered apps
        let apps = Self::load_apps(&app_dir)?;

        Ok(Self {
            app_dir,
            apps: Arc::new(Mutex::new(apps)),
            keypair,
        })
    }

    /// Load registered apps from the app directory
    fn load_apps(app_dir: &Path) -> Result<HashMap<String, AppRegistration>> {
        let mut apps = HashMap::new();

        // Iterate over files in the app directory
        for entry in fs::read_dir(app_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip non-JSON files and the keypair file
            let is_not_json = match path.extension() {
                None => true,
                Some(ext) => ext != "json"
            };
            let is_keypair_file = match path.file_name() {
                None => true,
                Some(name) => name == "fold_client.key"
            };
            if is_not_json || is_keypair_file {
                continue;
            }

            // Load the app registration
            let file = File::open(&path)?;
            let app: AppRegistration = serde_json::from_reader(file)
                .map_err(|e| FoldClientError::Auth(format!("Failed to parse app registration: {}", e)))?;

            // Add the app to the map
            apps.insert(app.app_id.clone(), app);
        }

        Ok(apps)
    }

    /// Register a new app
    pub fn register_app(&self, app_name: &str, permissions: &[&str]) -> Result<AppRegistration> {
        // Generate a unique app ID and token
        let app_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();

        // Generate a keypair for the app
        let mut csprng = OsRng07{};
        let app_keypair = Keypair::generate(&mut csprng);
        let public_key_bytes = app_keypair.public.to_bytes();
        let public_key = base64::encode(public_key_bytes);

        // Create the app registration
        let app = AppRegistration {
            app_id: app_id.clone(),
            app_name: app_name.to_string(),
            token: token.clone(),
            permissions: permissions.iter().map(|&s| s.to_string()).collect(),
            public_key,
            created_at: Utc::now(),
        };

        // Save the app registration
        let app_path = self.app_dir.join(format!("{}.json", app_id));
        let file = File::create(&app_path)?;
        serde_json::to_writer_pretty(file, &app)
            .map_err(|e| FoldClientError::Auth(format!("Failed to save app registration: {}", e)))?;

        // Save the app keypair
        let keypair_path = self.app_dir.join(format!("{}.key", app_id));
        let mut file = File::create(&keypair_path)?;
        file.write_all(&app_keypair.to_bytes())?;

        // Add the app to the map
        let mut apps = self.apps.lock().unwrap();
        apps.insert(app_id.clone(), app.clone());

        Ok(app)
    }

    /// Get an app registration by ID
    pub fn get_app(&self, app_id: &str) -> Result<AppRegistration> {
        let apps = self.apps.lock().unwrap();
        apps.get(app_id)
            .cloned()
            .ok_or_else(|| FoldClientError::Auth(format!("App not found: {}", app_id)))
    }

    /// Verify a request signature
    pub fn verify_signature(&self, app_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        // Get the app registration
        let app = self.get_app(app_id)?;

        // Decode the public key
        let public_key_bytes = base64::decode(&app.public_key)
            .map_err(|e| FoldClientError::Auth(format!("Failed to decode public key: {}", e)))?;
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| FoldClientError::Auth(format!("Invalid public key: {}", e)))?;

        // Decode the signature
        let signature = Signature::from_bytes(signature)
            .map_err(|e| FoldClientError::Auth(format!("Invalid signature: {}", e)))?;

        // Verify the signature
        match public_key.verify(message, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Sign a message with the FoldClient's private key
    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        let signature = self.keypair.sign(message);
        Ok(signature.to_bytes().to_vec())
    }

    /// Get the FoldClient's public key
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.public.to_bytes().to_vec()
    }

    /// Check if an app has a specific permission
    pub fn check_permission(&self, app_id: &str, permission: &str) -> Result<bool> {
        let app = self.get_app(app_id)?;
        Ok(app.permissions.contains(&permission.to_string()))
    }
    
    /// Verify an app token
    pub fn verify_app_token(&self, app_id: &str, token: &str) -> Result<bool> {
        let app = self.get_app(app_id)?;
        Ok(app.token == token)
    }
    
    /// List all registered apps
    pub fn list_apps(&self) -> Result<Vec<AppRegistration>> {
        let apps = self.apps.lock().unwrap();
        let app_list = apps.values().cloned().collect();
        Ok(app_list)
    }
}
