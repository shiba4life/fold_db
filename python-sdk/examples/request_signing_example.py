#!/usr/bin/env python3
"""
DataFold Python SDK - Request Signing Example

This example demonstrates how to use the DataFold Python SDK request signing
functionality to authenticate HTTP requests using RFC 9421 HTTP Message Signatures
with Ed25519 digital signatures.
"""

import json
import time
import sys
import os

# Add src to path for development
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from datafold_sdk import (
    # Key generation
    generate_key_pair,
    # Request signing
    create_signing_config,
    create_from_profile,
    RFC9421Signer,
    SignableRequest,
    HttpMethod,
    # HTTP integration
    create_signing_session,
    DataFoldHttpClient,
    ServerConfig,
    # Utilities
    generate_nonce,
    generate_timestamp,
    format_rfc3339_timestamp,
)


def basic_signing_example():
    """Demonstrate basic request signing workflow"""
    print("=== Basic Request Signing Example ===")
    
    # 1. Generate Ed25519 key pair
    print("1. Generating Ed25519 key pair...")
    key_pair = generate_key_pair()
    print(f"   Public key: {key_pair.public_key.hex()[:16]}...")
    print(f"   Private key: {key_pair.private_key.hex()[:16]}...")
    
    # 2. Create signing configuration
    print("\n2. Creating signing configuration...")
    config = (create_signing_config()
              .key_id("example-client-001")
              .private_key(key_pair.private_key)
              .profile("standard")  # Use standard security profile
              .build())
    
    print(f"   Key ID: {config.key_id}")
    print(f"   Algorithm: {config.algorithm}")
    print(f"   Components: method={config.components.method}, "
          f"target_uri={config.components.target_uri}, "
          f"content_digest={config.components.content_digest}")
    
    # 3. Create a sample request
    print("\n3. Creating sample HTTP request...")
    request = SignableRequest(
        method=HttpMethod.POST,
        url="https://api.datafold.com/api/crypto/keys/register",
        headers={
            "content-type": "application/json",
            "user-agent": "DataFold-Python-SDK-Example/1.0"
        },
        body=json.dumps({
            "client_id": "example-client-001",
            "public_key": key_pair.public_key.hex(),
            "key_name": "Example Client Key"
        })
    )
    
    print(f"   Method: {request.method}")
    print(f"   URL: {request.url}")
    print(f"   Headers: {request.headers}")
    print(f"   Body length: {len(request.body) if request.body else 0} bytes")
    
    # 4. Sign the request
    print("\n4. Signing the request...")
    signer = RFC9421Signer(config)
    
    start_time = time.perf_counter()
    result = signer.sign_request(request)
    end_time = time.perf_counter()
    
    signing_time_ms = (end_time - start_time) * 1000
    print(f"   Signing completed in {signing_time_ms:.2f}ms")
    
    # 5. Display results
    print("\n5. Signature results:")
    print(f"   Signature-Input: {result.signature_input}")
    print(f"   Signature: {result.signature}")
    print(f"   Additional headers: {list(result.headers.keys())}")
    
    print("\n6. Canonical message that was signed:")
    print("   " + "\n   ".join(result.canonical_message.split('\n')))
    
    return result


def security_profiles_example():
    """Demonstrate different security profiles"""
    print("\n\n=== Security Profiles Example ===")
    
    key_pair = generate_key_pair()
    
    profiles = ["minimal", "standard", "strict"]
    
    for profile_name in profiles:
        print(f"\n--- {profile_name.title()} Profile ---")
        
        config = create_from_profile(
            profile_name,
            f"client-{profile_name}",
            key_pair.private_key
        )
        
        # Create different request based on profile
        if profile_name == "minimal":
            # GET request for minimal profile
            request = SignableRequest(
                method=HttpMethod.GET,
                url="https://api.datafold.com/api/data/query?limit=10",
                headers={"accept": "application/json"}
            )
        else:
            # POST request for standard/strict profiles
            body = '{"data": "example"}'
            headers = {
                "content-type": "application/json",
                "authorization": "Bearer token123",
                "user-agent": "DataFold-SDK/1.0"
            }
            
            # Add content-length for strict profile
            if profile_name == "strict":
                headers["content-length"] = str(len(body.encode('utf-8')))
            
            request = SignableRequest(
                method=HttpMethod.POST,
                url="https://api.datafold.com/api/data/create",
                headers=headers,
                body=body
            )
        
        signer = RFC9421Signer(config)
        result = signer.sign_request(request)
        
        print(f"Key ID: {config.key_id}")
        print(f"Components: {config.components.__dict__}")
        print(f"Canonical message lines: {len(result.canonical_message.split())}")
        print(f"Signature: {result.signature[:50]}...")


def http_integration_example():
    """Demonstrate HTTP client integration"""
    print("\n\n=== HTTP Client Integration Example ===")
    
    key_pair = generate_key_pair()
    signing_config = create_from_profile(
        "standard",
        "http-example-client",
        key_pair.private_key
    )
    
    # Method 1: Signing Session
    print("\n1. Using SigningSession:")
    session = create_signing_session(signing_config)
    
    print(f"   Signing enabled: {session.auto_sign}")
    print(f"   Key ID: {session.signing_config.key_id}")
    
    # Method 2: DataFold HTTP Client with signing
    print("\n2. Using DataFoldHttpClient with signing:")
    server_config = ServerConfig(
        base_url="https://api.datafold.com",
        timeout=30.0
    )
    
    # Note: In real usage, you would actually make requests
    print(f"   Server: {server_config.base_url}")
    print("   Signing would be automatically applied to all requests")
    
    # Demonstrate signing a prepared request manually
    print("\n3. Manual request signing:")
    manual_request = SignableRequest(
        method=HttpMethod.PUT,
        url="https://api.datafold.com/api/resource/123",
        headers={"content-type": "application/json"},
        body='{"updated": true}'
    )
    
    signer = RFC9421Signer(signing_config)
    signed_result = signer.sign_request(manual_request)
    
    print(f"   Original headers: {manual_request.headers}")
    print(f"   Additional signature headers: {list(signed_result.headers.keys())}")


def performance_benchmark():
    """Demonstrate signing performance"""
    print("\n\n=== Performance Benchmark ===")
    
    key_pair = generate_key_pair()
    config = create_from_profile("standard", "perf-test", key_pair.private_key)
    signer = RFC9421Signer(config)
    
    # Test different request sizes
    test_cases = [
        ("Small request", '{"test": true}'),
        ("Medium request", json.dumps({"data": "x" * 1000})),
        ("Large request", json.dumps({"data": "x" * 10000}))
    ]
    
    for name, body in test_cases:
        request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={"content-type": "application/json"},
            body=body
        )
        
        # Time multiple iterations
        iterations = 100
        start_time = time.perf_counter()
        
        for _ in range(iterations):
            signer.sign_request(request)
        
        end_time = time.perf_counter()
        avg_time_ms = ((end_time - start_time) / iterations) * 1000
        
        print(f"{name:15s}: {avg_time_ms:.2f}ms avg (body: {len(body)} bytes)")


def utilities_example():
    """Demonstrate utility functions"""
    print("\n\n=== Utility Functions Example ===")
    
    print("1. Nonce generation:")
    for i in range(3):
        nonce = generate_nonce()
        print(f"   Nonce {i+1}: {nonce}")
    
    print("\n2. Timestamp utilities:")
    timestamp = generate_timestamp()
    rfc3339_time = format_rfc3339_timestamp(timestamp)
    print(f"   Unix timestamp: {timestamp}")
    print(f"   RFC 3339 format: {rfc3339_time}")
    
    print("\n3. URL parsing:")
    from datafold_sdk.signing.utils import parse_url
    
    test_urls = [
        "https://api.example.com/simple",
        "https://api.example.com/path?param=value",
        "http://localhost:8080/api/test"
    ]
    
    for url in test_urls:
        parsed = parse_url(url)
        print(f"   {url}")
        print(f"     -> target-uri: {parsed['target_uri']}")
    
    print("\n4. Content digest:")
    from datafold_sdk.signing.utils import calculate_content_digest
    from datafold_sdk.signing.types import DigestAlgorithm
    
    content = "Hello, World!"
    digest = calculate_content_digest(content, DigestAlgorithm.SHA256)
    print(f"   Content: {content}")
    print(f"   SHA-256 digest: {digest.header_value}")


def error_handling_example():
    """Demonstrate error handling"""
    print("\n\n=== Error Handling Example ===")
    
    key_pair = generate_key_pair()
    
    print("1. Configuration errors:")
    try:
        # Missing key ID
        create_signing_config().private_key(key_pair.private_key).build()
    except Exception as e:
        print(f"   Missing key ID: {type(e).__name__}: {e}")
    
    try:
        # Invalid private key
        create_signing_config().key_id("test").private_key(b"invalid").build()
    except Exception as e:
        print(f"   Invalid private key: {type(e).__name__}: {e}")
    
    print("\n2. Request errors:")
    config = create_from_profile("standard", "test", key_pair.private_key)
    signer = RFC9421Signer(config)
    
    try:
        # Invalid URL
        bad_request = SignableRequest(
            method=HttpMethod.GET,
            url="not-a-url",
            headers={}
        )
        signer.sign_request(bad_request)
    except Exception as e:
        print(f"   Invalid URL: {type(e).__name__}: {e}")
    
    try:
        # Missing required header
        config_with_auth = (create_signing_config()
                           .key_id("test")
                           .private_key(key_pair.private_key)
                           .headers(["authorization"])
                           .build())
        
        auth_signer = RFC9421Signer(config_with_auth)
        request_without_auth = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={"content-type": "application/json"},
            body='{"test": true}'
        )
        auth_signer.sign_request(request_without_auth)
    except Exception as e:
        print(f"   Missing required header: {type(e).__name__}: {e}")


def main():
    """Run all examples"""
    print("DataFold Python SDK - Request Signing Examples")
    print("=" * 50)
    
    try:
        # Run examples
        basic_signing_example()
        security_profiles_example()
        http_integration_example()
        performance_benchmark()
        utilities_example()
        error_handling_example()
        
        print("\n\n=== All Examples Completed Successfully! ===")
        print("\nNext steps:")
        print("1. Register your public key with the DataFold server")
        print("2. Configure your application with the signing configuration")
        print("3. Use SigningSession or DataFoldHttpClient for automatic signing")
        print("4. Monitor performance and adjust security profiles as needed")
        
    except Exception as e:
        print(f"\n❌ Example failed: {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()