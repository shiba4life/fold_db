//! Authentication command handlers
//! 
//! This module contains handlers for all authentication-related operations
//! including profile management, key registration, and server integration.

use crate::cli::args::{CliSecurityLevel, HttpMethod, MessageEncoding, ProfileAction};
use crate::cli::auth::{CliAuthProfile, CliSigningConfig};
use crate::cli::config::{CliConfigManager, ServerConfig};
use crate::cli::http_client::{HttpClientBuilder, RetryConfig};
use crate::cli::signing_config::SigningMode;
use crate::cli::utils::key_utils::{
    decrypt_key, encrypt_key, ensure_storage_dir, get_default_storage_dir, get_secure_passphrase,
    handle_retrieve_key_internal, KeyStorageConfig,
};
use crate::crypto::ed25519::generate_master_keypair;
use crate::crypto::{Argon2Params, MasterKeyPair};
use base64::{engine::general_purpose, Engine as _};
use log::info;
use reqwest::Client;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

/// Public key registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyRegistrationRequest {
    client_id: Option<String>,
    user_id: Option<String>,
    public_key: String,
    key_name: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

/// Public key registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyRegistrationResponse {
    registration_id: String,
    client_id: String,
    public_key: String,
    key_name: Option<String>,
    registered_at: String,
    status: String,
}

/// Public key status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyStatusResponse {
    registration_id: String,
    client_id: String,
    public_key: String,
    key_name: Option<String>,
    registered_at: String,
    status: String,
    last_used: Option<String>,
}

/// Signature verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationRequest {
    client_id: String,
    message: String,
    signature: String,
    message_encoding: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

/// Signature verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationResponse {
    verified: bool,
    client_id: String,
    public_key: String,
    verified_at: String,
    message_hash: String,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

/// API error structure from server responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    code: String,
    message: String,
    details: HashMap<String, serde_json::Value>,
}

/// Create HTTP client with retry and timeout configuration
fn create_http_client(timeout_secs: u64) -> Result<Client, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("datafold-cli/0.1.0")
        .build()?;
    Ok(client)
}

/// Perform HTTP request with retry logic
async fn http_request_with_retry<T>(
    _client: &Client,
    request_builder: reqwest::RequestBuilder,
    retries: u32,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let mut last_error = None;

    for attempt in 0..=retries {
        if attempt > 0 {
            println!("Retrying request (attempt {}/{})", attempt + 1, retries + 1);
            tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
        }

        // Clone the request builder for each attempt
        let request = match request_builder.try_clone() {
            Some(req) => req,
            None => return Err("Failed to clone request for retry".into()),
        };

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let response_text = response.text().await?;

                if status.is_success() {
                    // Parse the API response wrapper
                    let api_response: ApiResponse<T> = serde_json::from_str(&response_text)
                        .map_err(|e| format!("Failed to parse response: {}", e))?;

                    if api_response.success {
                        if let Some(data) = api_response.data {
                            return Ok(data);
                        } else {
                            return Err(
                                "API response marked as successful but contained no data".into()
                            );
                        }
                    } else if let Some(error) = api_response.error {
                        return Err(format!("API error: {} - {}", error.code, error.message).into());
                    } else {
                        return Err(
                            "API response marked as failed but contained no error details".into(),
                        );
                    }
                } else {
                    let error_msg = format!("HTTP error {}: {}", status, response_text);
                    last_error = Some(error_msg.clone().into());

                    // Don't retry on client errors (4xx), only server errors (5xx) and network issues
                    if status.is_client_error() {
                        return Err(error_msg.into());
                    }

                    println!("Server error, will retry: {}", error_msg);
                }
            }
            Err(e) => {
                let error_msg = format!("Network error: {}", e);
                last_error = Some(error_msg.clone().into());
                println!("Network error, will retry: {}", error_msg);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "All retry attempts failed".into()))
}

/// Convert HttpMethod enum to string
pub fn method_to_string(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
    }
}

/// Handle CLI authentication initialization
pub fn handle_auth_init(
    server_url: String,
    profile: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    user_id: Option<String>,
    _environment: Option<String>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Initializing CLI authentication...");

    // Load or create CLI configuration
    let mut config_manager = CliConfigManager::new()?;

    // Check if profile already exists
    if config_manager.get_profile(&profile).is_some() && !force {
        return Err(format!(
            "Profile '{}' already exists. Use --force to overwrite.",
            profile
        )
        .into());
    }

    // Verify the key exists in storage
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if !key_file.exists() {
        return Err(format!(
            "Key '{}' not found in storage. Use 'auth-keygen' or 'store-key' to create it first.",
            key_id
        )
        .into());
    }

    // Generate client ID
    let client_id = format!("cli_{}", uuid::Uuid::new_v4());

    // Create authentication profile
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "datafold-cli".to_string());
    metadata.insert("created_by".to_string(), "auth-init".to_string());

    let auth_profile = CliAuthProfile {
        client_id: client_id.clone(),
        key_id: key_id.clone(),
        user_id,
        server_url: server_url.clone(),
        metadata,
    };

    // Add profile to configuration
    config_manager.add_profile(profile.clone(), auth_profile)?;
    config_manager.save()?;

    println!(
        "✅ Authentication profile '{}' created successfully!",
        profile
    );
    println!("Client ID: {}", client_id);
    println!("Key ID: {}", key_id);
    println!("Server URL: {}", server_url);

    if config_manager.config().default_profile.as_ref() == Some(&profile) {
        println!("✨ Set as default profile");
    }

    println!("\n💡 Next steps:");
    println!(
        "1. Register your public key: datafold register-key --key-id {} --client-id {}",
        key_id, client_id
    );
    println!("2. Test authentication: datafold auth-test");

    Ok(())
}

/// Handle CLI authentication status
pub fn handle_auth_status(
    verbose: bool,
    profile: Option<String>,
    _environment: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 CLI Authentication Status");
    println!("============================");

    let config_manager = CliConfigManager::new()?;
    let status = config_manager.auth_status();

    if !status.configured {
        println!("❌ Authentication not configured");
        println!("\n💡 To get started:");
        println!("1. Generate a key: datafold auth-keygen --key-id my-key");
        println!("2. Initialize auth: datafold auth-init --key-id my-key --server-url https://your-server.com");
        return Ok(());
    }

    println!("✅ Authentication configured");

    if let Some(profile_name) = &profile {
        // Show specific profile
        if let Some(prof) = config_manager.get_profile(profile_name) {
            println!("\n📋 Profile: {}", profile_name);
            println!("   Client ID: {}", prof.client_id);
            println!("   Key ID: {}", prof.key_id);
            println!("   Server URL: {}", prof.server_url);

            if let Some(user_id) = &prof.user_id {
                println!("   User ID: {}", user_id);
            }

            if verbose {
                println!("   Metadata:");
                for (key, value) in &prof.metadata {
                    println!("     {}: {}", key, value);
                }
            }
        } else {
            return Err(format!("Profile '{}' not found", profile_name).into());
        }
    } else {
        // Show default profile and overall status
        if let Some(client_id) = &status.client_id {
            println!("   Client ID: {}", client_id);
        }
        if let Some(key_id) = &status.key_id {
            println!("   Key ID: {}", key_id);
        }
        if let Some(server_url) = &status.server_url {
            println!("   Server URL: {}", server_url);
        }

        if verbose {
            println!("\n📋 All Profiles:");
            let profiles = config_manager.list_profiles();
            if profiles.is_empty() {
                println!("   No profiles configured");
            } else {
                for profile_name in profiles {
                    let is_default =
                        config_manager.config().default_profile.as_ref() == Some(profile_name);
                    let marker = if is_default { " (default)" } else { "" };
                    println!("   • {}{}", profile_name, marker);

                    if let Some(prof) = config_manager.get_profile(profile_name) {
                        println!("     Client ID: {}", prof.client_id);
                        println!("     Key ID: {}", prof.key_id);
                        println!("     Server: {}", prof.server_url);
                    }
                }
            }
        }
    }

    println!(
        "\n🔧 Configuration file: {}",
        config_manager.config_path().display()
    );

    Ok(())
}

/// Handle authentication profile management
pub async fn handle_auth_profile(action: ProfileAction) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = CliConfigManager::new()?;

    match action {
        ProfileAction::Create {
            name,
            server_url,
            key_id,
            user_id,
            set_default,
        } => {
            // Check if profile already exists
            if config_manager.get_profile(&name).is_some() {
                return Err(format!("Profile '{}' already exists", name).into());
            }

            // Generate client ID
            let client_id = format!("cli_{}", uuid::Uuid::new_v4());

            // Create profile
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "datafold-cli".to_string());
            metadata.insert("created_by".to_string(), "profile-create".to_string());

            let profile = CliAuthProfile {
                client_id: client_id.clone(),
                key_id: key_id.clone(),
                user_id,
                server_url: server_url.clone(),
                metadata,
            };

            config_manager.add_profile(name.clone(), profile)?;

            if set_default {
                config_manager.set_default_profile(name.clone())?;
            }

            config_manager.save()?;

            println!("✅ Profile '{}' created successfully!", name);
            println!("   Client ID: {}", client_id);
            println!("   Key ID: {}", key_id);
            println!("   Server URL: {}", server_url);

            if set_default {
                println!("   ✨ Set as default profile");
            }
        }

        ProfileAction::List { verbose } => {
            let profiles = config_manager.list_profiles();

            if profiles.is_empty() {
                println!("No authentication profiles configured");
                return Ok(());
            }

            println!("📋 Authentication Profiles:");
            println!("===========================");

            for profile_name in profiles {
                let is_default =
                    config_manager.config().default_profile.as_ref() == Some(profile_name);
                let marker = if is_default { " (default)" } else { "" };

                println!("\n• {}{}", profile_name, marker);

                if let Some(prof) = config_manager.get_profile(profile_name) {
                    println!("  Client ID: {}", prof.client_id);
                    println!("  Key ID: {}", prof.key_id);
                    println!("  Server: {}", prof.server_url);

                    if let Some(user_id) = &prof.user_id {
                        println!("  User ID: {}", user_id);
                    }

                    if verbose {
                        println!("  Metadata:");
                        for (key, value) in &prof.metadata {
                            println!("    {}: {}", key, value);
                        }
                    }
                }
            }
        }

        ProfileAction::Show { name } => {
            if let Some(prof) = config_manager.get_profile(&name) {
                let is_default = config_manager.config().default_profile.as_ref() == Some(&name);

                println!("📋 Profile: {}", name);
                if is_default {
                    println!("   Status: Default profile");
                }
                println!("   Client ID: {}", prof.client_id);
                println!("   Key ID: {}", prof.key_id);
                println!("   Server URL: {}", prof.server_url);

                if let Some(user_id) = &prof.user_id {
                    println!("   User ID: {}", user_id);
                }

                println!("   Metadata:");
                for (key, value) in &prof.metadata {
                    println!("     {}: {}", key, value);
                }
            } else {
                return Err(format!("Profile '{}' not found", name).into());
            }
        }

        ProfileAction::Update {
            name,
            server_url,
            key_id,
            user_id,
        } => {
            if let Some(mut prof) = config_manager.get_profile(&name).cloned() {
                let mut updated = false;

                if let Some(new_url) = server_url {
                    prof.server_url = new_url;
                    updated = true;
                }

                if let Some(new_key_id) = key_id {
                    prof.key_id = new_key_id;
                    updated = true;
                }

                if let Some(new_user_id) = user_id {
                    prof.user_id = Some(new_user_id);
                    updated = true;
                }

                if updated {
                    config_manager.add_profile(name.clone(), prof)?;
                    config_manager.save()?;
                    println!("✅ Profile '{}' updated successfully!", name);
                } else {
                    println!("ℹ️  No changes specified for profile '{}'", name);
                }
            } else {
                return Err(format!("Profile '{}' not found", name).into());
            }
        }

        ProfileAction::Delete { name, force } => {
            if config_manager.get_profile(&name).is_none() {
                return Err(format!("Profile '{}' not found", name).into());
            }

            if !force {
                print!(
                    "Are you sure you want to delete profile '{}'? (y/N): ",
                    name
                );
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().to_lowercase().starts_with('y') {
                    println!("❌ Cancelled");
                    return Ok(());
                }
            }

            config_manager.remove_profile(&name)?;
            config_manager.save()?;

            println!("✅ Profile '{}' deleted successfully!", name);
        }

        ProfileAction::SetDefault { name } => {
            if config_manager.get_profile(&name).is_none() {
                return Err(format!("Profile '{}' not found", name).into());
            }

            config_manager.set_default_profile(name.clone())?;
            config_manager.save()?;

            println!("✅ Profile '{}' set as default!", name);
        }
    }

    Ok(())
}

/// Handle CLI authentication key generation
pub async fn handle_auth_keygen(
    key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    force: bool,
    auto_register: bool,
    server_url: Option<String>,
    passphrase: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Generating authentication key pair...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    // Check if key already exists
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if key_file.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite.", key_id).into());
    }

    // Generate key pair
    let master_keypair = generate_master_keypair()?;

    // Get passphrase for key encryption
    let _passphrase = match passphrase {
        Some(p) => p,
        None => {
            print!("Enter passphrase to encrypt the key: ");
            io::stdout().flush()?;
            get_secure_passphrase()?
        }
    };

    // Store the key using the existing key storage infrastructure
    let private_key_bytes = master_keypair.secret_key_bytes();
    
    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Encrypt the key
    let storage_config = encrypt_key(&private_key_bytes, &_passphrase, &argon2_params)?;

    // Write encrypted key to file
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file, config_json)
        .map_err(|e| format!("Failed to write key file: {}", e))?;

    // Set file permissions to 600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&key_file)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_file, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
    }

    println!("✅ Key pair generated and stored!");
    println!("   Key ID: {}", key_id);
    println!(
        "   Public Key: {}",
        hex::encode(master_keypair.public_key_bytes())
    );
    println!("   Storage: {}", key_file.display());

    // Auto-register if requested
    if auto_register {
        if let Some(server) = server_url {
            println!("\n🔄 Auto-registering key with server...");

            let client_id = format!("cli_{}", uuid::Uuid::new_v4());

            match handle_register_key(
                server,
                key_id.clone(),
                Some(storage_dir),
                Some(client_id.clone()),
                None,
                Some(format!("CLI Key: {}", key_id)),
                30, // timeout
                3,  // retries
            )
            .await
            {
                Ok(()) => {
                    println!("✅ Key registered successfully!");
                    println!("   Client ID: {}", client_id);
                    println!("\n💡 To use this key for authentication:");
                    println!(
                        "   datafold auth-init --key-id {} --server-url <server-url>",
                        key_id
                    );
                }
                Err(e) => {
                    println!(
                        "⚠️  Key generation successful but registration failed: {}",
                        e
                    );
                    println!("   You can register manually later with:");
                    println!("   datafold register-key --key-id {}", key_id);
                }
            }
        } else {
            println!("\n💡 To register this key:");
            println!(
                "   datafold register-key --key-id {} --server-url <server-url>",
                key_id
            );
        }
    } else {
        println!("\n💡 Next steps:");
        println!("1. Register key: datafold register-key --key-id {}", key_id);
        println!("2. Initialize auth: datafold auth-init --key-id {}", key_id);
    }

    Ok(())
}

/// Handle authenticated request test
pub async fn handle_auth_test(
    endpoint: String,
    profile: Option<String>,
    method: HttpMethod,
    payload: Option<String>,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing authenticated request...");

    // Load CLI configuration
    let config_manager = CliConfigManager::new()?;

    // Get profile to use
    let auth_profile = if let Some(profile_name) = &profile {
        config_manager
            .get_profile(profile_name)
            .ok_or_else(|| format!("Profile '{}' not found", profile_name))?
    } else {
        config_manager
            .get_default_profile()
            .ok_or("No default profile configured. Use 'auth-init' to set up authentication.")?
    };

    // Get storage directory and load key
    let storage_dir = CliConfigManager::default_keys_dir()?;
    let key_file = storage_dir.join(format!("{}.json", auth_profile.key_id));

    if !key_file.exists() {
        return Err(format!("Key '{}' not found in storage", auth_profile.key_id).into());
    }

    // Load and decrypt the key
    let key_content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&key_content)?;

    print!("Enter passphrase to unlock key: ");
    io::stdout().flush()?;
    let passphrase = get_secure_passphrase()?;

    let private_key_bytes = decrypt_key(&key_config, &passphrase)?;
    let master_keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)?;

    // Create authenticated HTTP client
    let signing_config = CliSigningConfig::default();
    let client = HttpClientBuilder::new()
        .timeout_secs(timeout)
        .build_authenticated(master_keypair, auth_profile.clone(), Some(signing_config))?;

    // Build request URL
    let full_url = if endpoint.starts_with("http") {
        endpoint.to_string()
    } else {
        let path = if endpoint.starts_with('/') {
            endpoint.to_string()
        } else {
            format!("/{}", endpoint)
        };
        format!("{}{}", auth_profile.server_url.trim_end_matches('/'), path)
    };

    println!(
        "Making {} request to: {}",
        method_to_string(&method),
        full_url
    );

    // Execute request based on method
    let response = match method {
        HttpMethod::Get => client.get(&full_url).await?,
        HttpMethod::Post => {
            let body = payload.unwrap_or_else(|| "{}".to_string());
            client
                .post_json(
                    &full_url,
                    &serde_json::from_str::<serde_json::Value>(&body)?,
                )
                .await?
        }
        HttpMethod::Put => {
            let body = payload.unwrap_or_else(|| "{}".to_string());
            client
                .put_json(
                    &full_url,
                    &serde_json::from_str::<serde_json::Value>(&body)?,
                )
                .await?
        }
        HttpMethod::Patch => {
            let body = payload.unwrap_or_else(|| "{}".to_string());
            client
                .patch_json(
                    &full_url,
                    &serde_json::from_str::<serde_json::Value>(&body)?,
                )
                .await?
        }
        HttpMethod::Delete => client.delete(&full_url).await?,
    };

    // Display response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await?;

    println!("\n📄 Response:");
    println!("   Status: {}", status);

    if status.is_success() {
        println!("   ✅ Request successful!");
    } else {
        println!("   ❌ Request failed!");
    }

    println!("   Headers:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            println!("     {}: {}", name, value_str);
        }
    }

    println!("   Body:");
    if body.is_empty() {
        println!("     (empty)");
    } else {
        // Try to pretty-print JSON, otherwise show as-is
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(json) => {
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            Err(_) => {
                println!("{}", body);
            }
        }
    }

    Ok(())
}

/// Handle public key registration with server
#[allow(clippy::too_many_arguments)]
pub async fn handle_register_key(
    server_url: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    client_id: Option<String>,
    user_id: Option<String>,
    key_name: Option<String>,
    timeout: u64,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Registering public key with DataFold server...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    // Load the key from storage
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if !key_file.exists() {
        return Err(format!(
            "Key '{}' not found in storage. Use store-key to create it first.",
            key_id
        )
        .into());
    }

    // Read and decrypt the stored key
    let key_content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&key_content)?;

    print!("Enter passphrase to unlock key: ");
    io::stdout().flush()?;
    let passphrase = get_secure_passphrase()?;

    let private_key_bytes = decrypt_key(&key_config, &passphrase)?;

    // Extract public key from private key
    let master_keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)?;
    let public_key = master_keypair.public_key();
    let public_key_hex = hex::encode(public_key.to_bytes());

    // Generate client ID if not provided
    let client_id = client_id.unwrap_or_else(|| format!("cli_{}", uuid::Uuid::new_v4()));

    // Create HTTP client
    let client = create_http_client(timeout)?;

    // Prepare registration request
    let registration_request = PublicKeyRegistrationRequest {
        client_id: Some(client_id.clone()),
        user_id,
        public_key: public_key_hex,
        key_name,
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "datafold-cli".to_string());
            meta.insert("key_id".to_string(), key_id.clone());
            meta
        }),
    };

    // Send registration request
    let register_url = format!(
        "{}/api/crypto/keys/register",
        server_url.trim_end_matches('/')
    );
    let request = client
        .post(&register_url)
        .header("Content-Type", "application/json")
        .json(&registration_request);

    println!("Sending registration request to: {}", register_url);
    let response: PublicKeyRegistrationResponse =
        http_request_with_retry(&client, request, retries).await?;

    println!("✅ Public key registered successfully!");
    println!("Registration ID: {}", response.registration_id);
    println!("Client ID: {}", response.client_id);
    println!("Status: {}", response.status);
    println!("Registered at: {}", response.registered_at);

    // Save client ID for future use
    let client_file = storage_dir.join(format!("{}_client.json", key_id));
    let client_info = json!({
        "client_id": response.client_id,
        "registration_id": response.registration_id,
        "server_url": server_url,
        "registered_at": response.registered_at
    });
    fs::write(client_file, serde_json::to_string_pretty(&client_info)?)?;

    println!("Client information saved for future use");

    Ok(())
}

/// Handle checking public key registration status
pub async fn handle_check_registration(
    server_url: String,
    client_id: String,
    timeout: u64,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking public key registration status...");

    // Create HTTP client
    let client = create_http_client(timeout)?;

    // Send status request
    let status_url = format!(
        "{}/api/crypto/keys/status/{}",
        server_url.trim_end_matches('/'),
        client_id
    );
    let request = client.get(&status_url);

    println!("Requesting status from: {}", status_url);
    let response: PublicKeyStatusResponse =
        http_request_with_retry(&client, request, retries).await?;

    println!("✅ Registration status retrieved successfully!");
    println!("Registration ID: {}", response.registration_id);
    println!("Client ID: {}", response.client_id);
    println!("Public Key: {}", response.public_key);
    println!("Status: {}", response.status);
    println!("Registered at: {}", response.registered_at);
    if let Some(last_used) = response.last_used {
        println!("Last used: {}", last_used);
    } else {
        println!("Last used: Never");
    }
    if let Some(key_name) = response.key_name {
        println!("Key name: {}", key_name);
    }

    Ok(())
}

/// Handle signing message and verifying with server
#[allow(clippy::too_many_arguments)]
pub async fn handle_sign_and_verify(
    server_url: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    client_id: String,
    message: Option<String>,
    message_file: Option<PathBuf>,
    message_encoding: MessageEncoding,
    timeout: u64,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Signing message and verifying with server...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    // Load the key from storage
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if !key_file.exists() {
        return Err(format!(
            "Key '{}' not found in storage. Use store-key to create it first.",
            key_id
        )
        .into());
    }

    // Read and decrypt the stored key
    let key_content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&key_content)?;

    print!("Enter passphrase to unlock key: ");
    io::stdout().flush()?;
    let passphrase = get_secure_passphrase()?;

    let private_key_bytes = decrypt_key(&key_config, &passphrase)?;
    let master_keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)?;

    // Get message to sign
    let message_to_sign = match (message, message_file) {
        (Some(msg), None) => msg,
        (None, Some(file)) => fs::read_to_string(file)?,
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --message and --message-file".into());
        }
        (None, None) => {
            return Err("Must specify either --message or --message-file".into());
        }
    };

    // Sign the message
    let signature = master_keypair.sign_data(message_to_sign.as_bytes())?;
    let signature_hex = hex::encode(signature);

    println!("Message signed successfully");
    println!("Signature: {}", signature_hex);

    // Create HTTP client
    let client = create_http_client(timeout)?;

    // Prepare verification request
    let encoding_str = match message_encoding {
        MessageEncoding::Utf8 => "utf8",
        MessageEncoding::Hex => "hex",
        MessageEncoding::Base64 => "base64",
    };

    let verification_request = SignatureVerificationRequest {
        client_id: client_id.clone(),
        message: message_to_sign,
        signature: signature_hex,
        message_encoding: Some(encoding_str.to_string()),
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "datafold-cli".to_string());
            meta.insert("key_id".to_string(), key_id.clone());
            meta
        }),
    };

    // Send verification request
    let verify_url = format!(
        "{}/api/crypto/signatures/verify",
        server_url.trim_end_matches('/')
    );
    let request = client
        .post(&verify_url)
        .header("Content-Type", "application/json")
        .json(&verification_request);

    println!("Sending verification request to: {}", verify_url);
    let response: SignatureVerificationResponse =
        http_request_with_retry(&client, request, retries).await?;

    println!("✅ Signature verification completed!");
    println!(
        "Verified: {}",
        if response.verified {
            "✅ SUCCESS"
        } else {
            "❌ FAILED"
        }
    );
    println!("Client ID: {}", response.client_id);
    println!("Public Key: {}", response.public_key);
    println!("Verified at: {}", response.verified_at);
    println!("Message hash: {}", response.message_hash);

    if !response.verified {
        return Err("Signature verification failed".into());
    }

    Ok(())
}

/// Handle end-to-end server integration test
#[allow(clippy::too_many_arguments)]
pub async fn handle_test_server_integration(
    server_url: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    test_message: String,
    timeout: u64,
    retries: u32,
    security_level: CliSecurityLevel,
    cleanup: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Starting end-to-end server integration test...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    let test_key_id = format!("{}_test", key_id);
    let client_id = format!("test_{}", uuid::Uuid::new_v4());

    println!("Step 1: Generating test keypair...");
    // Generate a test key
    let master_keypair = generate_master_keypair()?;

    // Store the test key
    print!("Enter passphrase for test key: ");
    io::stdout().flush()?;
    let _passphrase = get_secure_passphrase()?;

    // Use the internal key storage functions
    let private_key_bytes = master_keypair.secret_key_bytes();
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    let storage_config = encrypt_key(&private_key_bytes, &_passphrase, &argon2_params)?;
    let key_file_path = storage_dir.join(format!("{}.json", test_key_id));
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&key_file_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_file_path, perms)?;
    }

    println!("✅ Test key generated and stored");

    println!("Step 2: Registering public key with server...");
    // Register the key
    match handle_register_key(
        server_url.clone(),
        test_key_id.clone(),
        Some(storage_dir.clone()),
        Some(client_id.clone()),
        None,
        Some("Integration Test Key".to_string()),
        timeout,
        retries,
    )
    .await
    {
        Ok(()) => println!("✅ Key registration successful"),
        Err(e) => {
            eprintln!("❌ Key registration failed: {}", e);
            if cleanup {
                let _ = fs::remove_file(&key_file_path);
            }
            return Err(e);
        }
    }

    println!("Step 3: Checking registration status...");
    // Check registration status
    match handle_check_registration(server_url.clone(), client_id.clone(), timeout, retries).await {
        Ok(()) => println!("✅ Registration status check successful"),
        Err(e) => {
            eprintln!("❌ Registration status check failed: {}", e);
            if cleanup {
                let _ = fs::remove_file(&key_file_path);
            }
            return Err(e);
        }
    }

    println!("Step 4: Signing and verifying message...");
    // Sign and verify
    match handle_sign_and_verify(
        server_url.clone(),
        test_key_id.clone(),
        Some(storage_dir.clone()),
        client_id.clone(),
        Some(test_message),
        None,
        MessageEncoding::Utf8,
        timeout,
        retries,
    )
    .await
    {
        Ok(()) => println!("✅ Message signing and verification successful"),
        Err(e) => {
            eprintln!("❌ Message signing and verification failed: {}", e);
            if cleanup {
                let _ = fs::remove_file(&key_file_path);
            }
            return Err(e);
        }
    }

    if cleanup {
        println!("Step 5: Cleaning up test key...");
        match fs::remove_file(&key_file_path) {
            Ok(()) => println!("✅ Test key cleaned up"),
            Err(e) => {
                eprintln!("⚠️ Failed to clean up test key: {}", e);
                // Don't fail the test for cleanup issues
            }
        }
    }

    println!("🎉 End-to-end server integration test completed successfully!");
    println!("All server integration functionality is working correctly.");

    Ok(())
}