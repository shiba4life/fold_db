//! Example demonstrating CLI signature verification utilities
//!
//! This example shows how to use the DataFold CLI verification commands
//! to verify signatures and analyze signature formats.

use datafold::cli::verification::{
    CliSignatureVerifier, CliVerificationConfig, SignatureInspector
};
use datafold::crypto::ed25519::generate_master_keypair;
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DataFold CLI Verification Example ===\n");

    // Generate a test keypair
    let keypair = generate_master_keypair()?;
    let public_key_bytes = keypair.public_key_bytes();
    let test_message = b"Hello, DataFold CLI verification!";

    // Sign the test message
    let signature_bytes = keypair.sign_data(test_message)?;
    let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature_bytes);

    println!("1. Generated test keypair and signed message");
    println!("   Message: {}", String::from_utf8_lossy(test_message));
    println!("   Public Key: {}", hex::encode(public_key_bytes));
    println!("   Signature: {}\n", signature_b64);

    // Create verifier with default configuration
    let mut config = CliVerificationConfig::default();
    let key_id = "test-key-1".to_string();
    
    // Add our public key to the verifier
    let mut verifier = CliSignatureVerifier::new(config);
    verifier.add_public_key(key_id.clone(), public_key_bytes.to_vec())?;

    println!("2. Created verifier and added public key");
    println!("   Key ID: {}", key_id);
    println!("   Available policies: {:?}\n", verifier.config().policies.keys().collect::<Vec<_>>());

    // Verify the message signature
    println!("3. Verifying message signature...");
    let result = verifier.verify_message_signature(
        test_message, 
        &signature_b64, 
        &key_id, 
        Some("default")
    ).await?;

    println!("   Status: {}", result.status);
    println!("   Signature Valid: {}", result.signature_valid);
    println!("   Total Time: {}ms", result.performance.total_time_ms);
    println!("   Algorithm: {}", result.diagnostics.signature_analysis.algorithm);
    
    // Show individual verification checks
    println!("\n   Individual Checks:");
    println!("   - Format Valid: {}", result.checks.format_valid);
    println!("   - Cryptographic Valid: {}", result.checks.cryptographic_valid);
    println!("   - Timestamp Valid: {}", result.checks.timestamp_valid);
    println!("   - Nonce Valid: {}", result.checks.nonce_valid);
    println!("   - Content Digest Valid: {}", result.checks.content_digest_valid);
    println!("   - Component Coverage Valid: {}", result.checks.component_coverage_valid);

    // Test signature inspection
    println!("\n4. Testing signature format inspection...");
    let inspector = SignatureInspector::new(true);
    
    // Create mock signature headers
    let mut headers = HashMap::new();
    headers.insert("signature-input".to_string(), 
                  "sig1=(\"@method\" \"@target-uri\");alg=\"ed25519\";created=1640995200".to_string());
    headers.insert("signature".to_string(), signature_b64.clone());
    headers.insert("content-digest".to_string(), 
                  "sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:".to_string());

    let analysis = inspector.inspect_signature_format(&headers);
    
    println!("   RFC 9421 Compliant: {}", analysis.is_valid_rfc9421);
    println!("   Signature Headers Found: {}", analysis.signature_headers.join(", "));
    println!("   Signature IDs: {}", analysis.signature_ids.join(", "));
    
    if !analysis.issues.is_empty() {
        println!("   Issues Found:");
        for issue in &analysis.issues {
            println!("   - {:?}: {} ({})", issue.severity, issue.message, issue.code);
        }
    } else {
        println!("   No format issues detected");
    }

    // Test different verification policies
    println!("\n5. Testing verification policies...");
    
    let policies = ["default", "strict", "permissive"];
    for policy_name in &policies {
        println!("   Testing policy: {}", policy_name);
        let policy_result = verifier.verify_message_signature(
            test_message, 
            &signature_b64, 
            &key_id, 
            Some(policy_name)
        ).await;
        
        match policy_result {
            Ok(result) => {
                println!("     Status: {} ({}ms)", result.status, result.performance.total_time_ms);
                println!("     Policy: {}", result.diagnostics.policy_compliance.policy_name);
            }
            Err(e) => {
                println!("     Error: {}", e);
            }
        }
    }

    // Generate diagnostic report
    println!("\n6. Generating diagnostic report...");
    let report = inspector.generate_diagnostic_report(&result);
    println!("{}", report);

    // Test invalid signature
    println!("\n7. Testing invalid signature detection...");
    let invalid_signature = "aW52YWxpZC1zaWduYXR1cmUtZGF0YQ=="; // "invalid-signature-data" in base64
    
    let invalid_result = verifier.verify_message_signature(
        test_message, 
        invalid_signature, 
        &key_id, 
        Some("default")
    ).await?;

    println!("   Invalid signature status: {}", invalid_result.status);
    println!("   Cryptographic valid: {}", invalid_result.checks.cryptographic_valid);
    
    if let Some(error) = &invalid_result.error {
        println!("   Error code: {}", error.code);
        println!("   Error message: {}", error.message);
    }

    println!("\n=== CLI Verification Example Complete ===");
    println!("\nTo use these features in the CLI:");
    println!("1. Generate a keypair: datafold generate-key --format hex");
    println!("2. Verify a signature: datafold verify-signature --message 'test' --signature '{}' --key-id 'test' --public-key '{}'", 
             signature_b64, hex::encode(public_key_bytes));
    println!("3. Inspect signature format: datafold inspect-signature --signature-input 'sig1=(\"@method\");alg=\"ed25519\"' --signature '{}'", signature_b64);
    println!("4. Configure verification: datafold verification-config show --policies");

    Ok(())
}