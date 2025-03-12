/// Verify the signature of a request using ECDSA with the P-256 curve
pub fn verify_signature(
    signature: &str,
    public_key: &str,
    message: &str,
) -> bool {
    // This is a simplified implementation of signature verification
    // using the ring crate for cryptographic operations
    
    use ring::signature;
    use base64::{Engine as _, engine::general_purpose};
    
    // Decode the base64-encoded public key and signature
    let public_key_bytes = match general_purpose::STANDARD.decode(public_key) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    let signature_bytes = match general_purpose::STANDARD.decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    // Create a message digest from the message
    let message_bytes = message.as_bytes();
    
    // Import the public key
    let public_key_alg = &signature::ECDSA_P256_SHA256_ASN1;
    
    // Create the UnparsedPublicKey
    let unparsed_public_key = signature::UnparsedPublicKey::new(public_key_alg, &public_key_bytes);
    
    // Verify the signature
    unparsed_public_key.verify(message_bytes, &signature_bytes).is_ok()
}
