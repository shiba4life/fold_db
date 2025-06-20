//! Simple test program to validate unified crypto functionality

use datafold::unified_crypto::{UnifiedCrypto, CryptoConfig};
use datafold::unified_crypto::primitives::{CryptoPrimitives, generate_master_keypair};
use datafold::unified_crypto::types::HashAlgorithm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing Unified Crypto Functionality");
    
    // Test 1: Initialize unified crypto
    println!("\n1. Initializing unified crypto system...");
    let config = CryptoConfig::default();
    let crypto = UnifiedCrypto::new(config)?;
    println!("✅ Unified crypto initialized successfully");
    
    // Test 2: Generate a keypair
    println!("\n2. Generating cryptographic keypair...");
    let keypair = crypto.generate_keypair()?;
    println!("✅ Keypair generated successfully");
    println!("   Key ID: {:?}", keypair.id());
    
    // Test 3: Test encryption and decryption
    println!("\n3. Testing encryption/decryption...");
    let plaintext = b"Hello, unified crypto!";
    let encrypted = crypto.encrypt(plaintext, keypair.public_key())?;
    println!("✅ Data encrypted successfully");
    println!("   Plaintext length: {} bytes", plaintext.len());
    println!("   Ciphertext length: {} bytes", encrypted.len());
    
    let decrypted = crypto.decrypt(&encrypted, keypair.private_key())?;
    println!("✅ Data decrypted successfully");
    
    if plaintext == &decrypted[..] {
        println!("✅ Encryption/decryption roundtrip successful");
    } else {
        println!("❌ Encryption/decryption roundtrip failed");
        return Err("Data integrity check failed".into());
    }
    
    // Test 4: Test signing and verification
    println!("\n4. Testing digital signatures...");
    let data_to_sign = b"Important message to sign";
    let signature = crypto.sign(data_to_sign, keypair.private_key())?;
    println!("✅ Digital signature created successfully");
    
    let is_valid = crypto.verify(data_to_sign, &signature, keypair.public_key())?;
    if is_valid {
        println!("✅ Signature verification successful");
    } else {
        println!("❌ Signature verification failed");
        return Err("Signature verification failed".into());
    }
    
    // Test 5: Test hashing
    println!("\n5. Testing cryptographic hashing...");
    let data_to_hash = b"Data to hash with SHA-256";
    let hash = crypto.hash(data_to_hash, HashAlgorithm::Sha256)?;
    println!("✅ Hash computed successfully");
    println!("   Input length: {} bytes", data_to_hash.len());
    println!("   Hash length: {} bytes", hash.len());
    
    // Test 6: Test legacy compatibility
    println!("\n6. Testing legacy compatibility...");
    let legacy_keypair = generate_master_keypair()?;
    println!("✅ Legacy keypair generation works");
    println!("   Legacy Key ID: {:?}", legacy_keypair.id());
    
    println!("\n🎉 All unified crypto tests passed successfully!");
    println!("\n📋 Summary:");
    println!("   ✅ Unified crypto initialization");
    println!("   ✅ Key pair generation");
    println!("   ✅ Encryption/decryption");
    println!("   ✅ Digital signatures");
    println!("   ✅ Cryptographic hashing");
    println!("   ✅ Legacy compatibility");
    
    println!("\n🔄 Migration adapters are functional and unified crypto is operational!");
    
    Ok(())
}