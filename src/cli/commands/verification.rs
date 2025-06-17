//! Signature verification command handlers
//! 
//! This module contains handlers for signature verification operations
//! including signature verification, inspection, and response verification.

use crate::cli::args::{HttpMethod, VerificationConfigAction, VerificationOutputFormat};
use crate::cli::utils::key_utils::parse_key_input;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Convert HttpMethod enum to string
fn method_to_string(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
    }
}

/// Handle signature verification command
#[allow(clippy::too_many_arguments)]
pub async fn handle_verify_signature(
    message: Option<String>,
    message_file: Option<PathBuf>,
    signature: String,
    key_id: String,
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
    policy: Option<String>,
    output_format: VerificationOutputFormat,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::verification::{CliSignatureVerifier, CliVerificationConfig};

    // Get message bytes
    let message_bytes = match (message, message_file) {
        (Some(msg), None) => msg.into_bytes(),
        (None, Some(file)) => fs::read(file)?,
        (Some(_), Some(_)) => return Err("Cannot specify both message and message-file".into()),
        (None, None) => return Err("Must specify either message or message-file".into()),
    };

    // Create verifier
    let config = CliVerificationConfig::default();

    // Add public key
    let public_key_bytes = match (public_key, public_key_file) {
        (Some(key), None) => parse_key_input(&key, false)?.to_vec(),
        (None, Some(file)) => {
            let key_str = fs::read_to_string(file)?;
            parse_key_input(key_str.trim(), false)?.to_vec()
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both public-key and public-key-file".into())
        }
        (None, None) => return Err("Must specify either public-key or public-key-file".into()),
    };

    let mut verifier = CliSignatureVerifier::new(config);
    verifier.add_public_key(key_id.clone(), public_key_bytes)?;

    // Perform verification
    let result = verifier
        .verify_message_signature(&message_bytes, &signature, &key_id, policy.as_deref())
        .await?;

    // Output result
    match output_format {
        VerificationOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        VerificationOutputFormat::Table => {
            println!("=== Signature Verification Result ===");
            println!("Status: {}", result.status);
            println!("Signature Valid: {}", result.signature_valid);
            println!("Total Time: {}ms", result.performance.total_time_ms);

            if debug {
                let inspector = crate::cli::verification::SignatureInspector::new(true);
                let report = inspector.generate_diagnostic_report(&result);
                println!("\n{}", report);
            }
        }
        VerificationOutputFormat::Compact => {
            println!(
                "{}: {}",
                result.status,
                if result.signature_valid { "✓" } else { "✗" }
            );
        }
    }

    if !result.signature_valid {
        std::process::exit(1);
    }

    Ok(())
}

/// Handle signature inspection command
pub async fn handle_inspect_signature(
    signature_input: Option<String>,
    signature: Option<String>,
    headers_file: Option<PathBuf>,
    output_format: VerificationOutputFormat,
    _detailed: bool,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::verification::SignatureInspector;

    let inspector = SignatureInspector::new(debug);
    let mut headers = HashMap::new();

    // Build headers from inputs
    if let Some(input) = signature_input {
        headers.insert("signature-input".to_string(), input);
    }
    if let Some(sig) = signature {
        headers.insert("signature".to_string(), sig);
    }
    if let Some(file) = headers_file {
        let content = fs::read_to_string(file)?;
        let file_headers: HashMap<String, String> = serde_json::from_str(&content)?;
        headers.extend(file_headers);
    }

    if headers.is_empty() {
        return Err(
            "Must provide signature headers via signature-input, signature, or headers-file".into(),
        );
    }

    // Inspect signature format
    let analysis = inspector.inspect_signature_format(&headers);

    // Output results
    match output_format {
        VerificationOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        VerificationOutputFormat::Table => {
            println!("=== Signature Format Analysis ===");
            println!("RFC 9421 Compliant: {}", analysis.is_valid_rfc9421);
            println!(
                "Signature Headers: {}",
                analysis.signature_headers.join(", ")
            );
            println!("Signature IDs: {}", analysis.signature_ids.join(", "));

            if !analysis.issues.is_empty() {
                println!("\n=== Issues Found ===");
                for issue in &analysis.issues {
                    println!("{:?}: {} - {}", issue.severity, issue.code, issue.message);
                    if let Some(component) = &issue.component {
                        println!("  Component: {}", component);
                    }
                }
            }
        }
        VerificationOutputFormat::Compact => {
            println!(
                "RFC9421: {} | Issues: {}",
                if analysis.is_valid_rfc9421 {
                    "✓"
                } else {
                    "✗"
                },
                analysis.issues.len()
            );
        }
    }

    Ok(())
}

/// Handle response verification command
#[allow(clippy::too_many_arguments)]
pub async fn handle_verify_response(
    url: String,
    method: HttpMethod,
    headers: Option<String>,
    body: Option<String>,
    body_file: Option<PathBuf>,
    key_id: String,
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
    policy: Option<String>,
    output_format: VerificationOutputFormat,
    debug: bool,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::verification::{CliSignatureVerifier, CliVerificationConfig};

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()?;

    // Build request
    let method_str = method_to_string(&method);
    let mut request_builder = match method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
        HttpMethod::Put => client.put(&url),
        HttpMethod::Patch => client.patch(&url),
        HttpMethod::Delete => client.delete(&url),
    };

    // Add headers if provided
    if let Some(headers_json) = headers {
        let headers_map: HashMap<String, String> = serde_json::from_str(&headers_json)?;
        for (key, value) in headers_map {
            request_builder = request_builder.header(key, value);
        }
    }

    // Add body if provided
    let request_body = match (body, body_file) {
        (Some(body_str), None) => Some(body_str),
        (None, Some(file)) => Some(fs::read_to_string(file)?),
        (Some(_), Some(_)) => return Err("Cannot specify both body and body-file".into()),
        (None, None) => None,
    };

    if let Some(body_content) = request_body {
        request_builder = request_builder.body(body_content);
    }

    // Send request
    let response = request_builder.send().await?;

    // Setup verifier
    let config = CliVerificationConfig::default();

    // Add public key
    let public_key_bytes = match (public_key, public_key_file) {
        (Some(key), None) => parse_key_input(&key, false)?.to_vec(),
        (None, Some(file)) => {
            let key_str = fs::read_to_string(file)?;
            parse_key_input(key_str.trim(), false)?.to_vec()
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both public-key and public-key-file".into())
        }
        (None, None) => return Err("Must specify either public-key or public-key-file".into()),
    };

    let mut verifier = CliSignatureVerifier::new(config);
    verifier.add_public_key(key_id.clone(), public_key_bytes)?;

    // Verify response
    let result = verifier
        .verify_response_with_context(&response, method_str, &url, policy.as_deref())
        .await?;

    // Output result
    match output_format {
        VerificationOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        VerificationOutputFormat::Table => {
            println!("=== Response Verification Result ===");
            println!("Status: {}", result.status);
            println!("Signature Valid: {}", result.signature_valid);
            println!("Total Time: {}ms", result.performance.total_time_ms);

            if debug {
                let inspector = crate::cli::verification::SignatureInspector::new(true);
                let report = inspector.generate_diagnostic_report(&result);
                println!("\n{}", report);
            }
        }
        VerificationOutputFormat::Compact => {
            println!(
                "{}: {}",
                result.status,
                if result.signature_valid { "✓" } else { "✗" }
            );
        }
    }

    if !result.signature_valid {
        std::process::exit(1);
    }

    Ok(())
}

/// Handle verification configuration command
pub async fn handle_verification_config(
    action: VerificationConfigAction,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::verification::{CliVerificationConfig, VerificationPolicy};

    match action {
        VerificationConfigAction::Show { policies, keys } => {
            let config = CliVerificationConfig::default();

            if policies {
                println!("=== Verification Policies ===");
                for (name, policy) in &config.policies {
                    println!("Policy: {}", name);
                    println!("  Description: {}", policy.description);
                    println!("  Verify Timestamp: {}", policy.verify_timestamp);
                    println!("  Verify Nonce: {}", policy.verify_nonce);
                    println!("  Verify Content Digest: {}", policy.verify_content_digest);
                    println!("  Required Components: {:?}", policy.required_components);
                    println!("  Allowed Algorithms: {:?}", policy.allowed_algorithms);
                    if name == &config.default_policy {
                        println!("  (Default Policy)");
                    }
                    println!();
                }
            }

            if keys {
                println!("=== Public Keys ===");
                if config.public_keys.is_empty() {
                    println!("No public keys configured");
                } else {
                    for (key_id, key_bytes) in &config.public_keys {
                        println!("Key ID: {}", key_id);
                        println!("  Length: {} bytes", key_bytes.len());
                        println!("  Fingerprint: {}", hex::encode(&key_bytes[..8]));
                        println!();
                    }
                }
            }

            if !policies && !keys {
                println!("=== Verification Configuration ===");
                println!("Default Policy: {}", config.default_policy);
                println!("Available Policies: {}", config.policies.len());
                println!("Configured Keys: {}", config.public_keys.len());
                println!(
                    "Performance Monitoring: {}",
                    config.performance_monitoring.enabled
                );
                println!("Debug Enabled: {}", config.debug.enabled);
            }
        }
        VerificationConfigAction::AddPolicy { name, config_file } => {
            let policy_json = fs::read_to_string(config_file)?;
            let _policy: VerificationPolicy = serde_json::from_str(&policy_json)?;
            println!(
                "Policy '{}' would be added (not implemented in this demo)",
                name
            );
        }
        VerificationConfigAction::RemovePolicy { name } => {
            println!(
                "Policy '{}' would be removed (not implemented in this demo)",
                name
            );
        }
        VerificationConfigAction::SetDefaultPolicy { name } => {
            println!(
                "Default policy would be set to '{}' (not implemented in this demo)",
                name
            );
        }
        VerificationConfigAction::AddPublicKey {
            key_id,
            public_key,
            public_key_file,
        } => {
            let _key_bytes = match (public_key, public_key_file) {
                (Some(key), None) => parse_key_input(&key, false)?.to_vec(),
                (None, Some(file)) => {
                    let key_str = fs::read_to_string(file)?;
                    parse_key_input(key_str.trim(), false)?.to_vec()
                }
                (Some(_), Some(_)) => {
                    return Err("Cannot specify both public-key and public-key-file".into())
                }
                (None, None) => {
                    return Err("Must specify either public-key or public-key-file".into())
                }
            };
            println!(
                "Public key '{}' would be added (not implemented in this demo)",
                key_id
            );
        }
        VerificationConfigAction::RemovePublicKey { key_id } => {
            println!(
                "Public key '{}' would be removed (not implemented in this demo)",
                key_id
            );
        }
        VerificationConfigAction::ListPublicKeys { verbose } => {
            let config = CliVerificationConfig::default();
            println!("=== Public Keys ===");
            if config.public_keys.is_empty() {
                println!("No public keys configured");
            } else {
                for (key_id, key_bytes) in &config.public_keys {
                    if verbose {
                        println!("Key ID: {}", key_id);
                        println!("  Length: {} bytes", key_bytes.len());
                        println!("  Fingerprint: {}", hex::encode(&key_bytes[..8]));
                        println!("  Full Key: {}", hex::encode(key_bytes));
                        println!();
                    } else {
                        println!("{} ({}B)", key_id, key_bytes.len());
                    }
                }
            }
        }
    }

    Ok(())
}