#!/usr/bin/env python3
"""
Basic test script to verify signing implementation works correctly
"""

import sys
import os
import traceback

# Add src to path so we can import the SDK
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

def test_basic_imports():
    """Test that all modules can be imported correctly"""
    print("Testing basic imports...")
    
    try:
        from datafold_sdk.signing.types import (
            SignableRequest, SigningConfig, HttpMethod, SignatureAlgorithm
        )
        print("✓ Types module imported successfully")
        
        from datafold_sdk.signing.utils import (
            generate_nonce, generate_timestamp, calculate_content_digest
        )
        print("✓ Utils module imported successfully")
        
        from datafold_sdk.signing.signing_config import create_signing_config
        print("✓ Configuration module imported successfully")
        
        from datafold_sdk.signing.rfc9421_signer import RFC9421Signer
        print("✓ Signer module imported successfully")
        
        
        
    except Exception as e:
        print(f"✗ Import failed: {e}")
        traceback.print_exc()
        return False


def test_basic_functionality():
    """Test basic signing functionality"""
    print("\nTesting basic functionality...")
    
    try:
        # Import required modules
        from datafold_sdk.crypto.ed25519 import generate_key_pair
        from datafold_sdk.signing import (
            create_signing_config, RFC9421Signer, SignableRequest, HttpMethod
        )
        
        # Generate key pair
        key_pair = generate_key_pair()
        print("✓ Generated Ed25519 key pair")
        
        # Create signing configuration
        config = (create_signing_config()
                 .key_id("test-client")
                 .private_key(key_pair.private_key)
                 .profile("standard")
                 .build())
        print("✓ Created signing configuration")
        
        # Create signer
        signer = RFC9421Signer(config)
        print("✓ Created RFC9421 signer")
        
        # Create test request
        request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={"content-type": "application/json"},
            body='{"test": true}'
        )
        print("✓ Created signable request")
        
        # Sign the request
        result = signer.sign_request(request)
        print("✓ Signed request successfully")
        
        # Verify result structure
        assert hasattr(result, 'signature_input')
        assert hasattr(result, 'signature')  
        assert hasattr(result, 'headers')
        assert hasattr(result, 'canonical_message')
        print("✓ Signature result has correct structure")
        
        # Verify signature format
        assert result.signature.startswith('sig1=:')
        assert result.signature.endswith(':')
        print("✓ Signature format is correct")
        
        # Verify headers
        assert 'signature-input' in result.headers
        assert 'signature' in result.headers
        assert 'content-digest' in result.headers
        print("✓ Required headers are present")
        
        # Verify canonical message
        lines = result.canonical_message.split('\n')
        assert len(lines) >= 3
        assert lines[-1].startswith('"@signature-params": ')
        print("✓ Canonical message format is correct")
        
        print(f"\nSignature result preview:")
        print(f"  Signature-Input: {result.signature_input[:80]}...")
        print(f"  Signature: {result.signature[:50]}...")
        print(f"  Canonical message lines: {len(lines)}")
        
        
        
    except Exception as e:
        print(f"✗ Functionality test failed: {e}")
        traceback.print_exc()
        assert False, f"Functionality test failed: {e}"


def test_utility_functions():
    """Test utility functions"""
    print("\nTesting utility functions...")
    
    try:
        from datafold_sdk.signing.utils import (
            generate_nonce, generate_timestamp, validate_nonce, 
            validate_timestamp, parse_url, calculate_content_digest
        )
        from datafold_sdk.signing.types import DigestAlgorithm
        
        # Test nonce generation
        nonce = generate_nonce()
        assert validate_nonce(nonce)
        print("✓ Nonce generation and validation")
        
        # Test timestamp generation
        timestamp = generate_timestamp()
        assert validate_timestamp(timestamp)
        print("✓ Timestamp generation and validation")
        
        # Test URL parsing
        url_parts = parse_url("https://api.example.com/path?param=value")
        assert url_parts['target_uri'] == "/path?param=value"
        print("✓ URL parsing")
        
        # Test content digest
        digest = calculate_content_digest("test content", DigestAlgorithm.SHA256)
        assert digest.algorithm == DigestAlgorithm.SHA256
        assert digest.header_value.startswith("sha-256=:")
        print("✓ Content digest calculation")
        
        
        
    except Exception as e:
        print(f"✗ Utility functions test failed: {e}")
        traceback.print_exc()
        assert False, f"Utility functions test failed: {e}"


def test_security_profiles():
    """Test security profiles"""
    print("\nTesting security profiles...")
    
    try:
        from datafold_sdk.crypto.ed25519 import generate_key_pair
        from datafold_sdk.signing import create_from_profile, SECURITY_PROFILES
        
        key_pair = generate_key_pair()
        
        # Test each profile
        for profile_name in SECURITY_PROFILES:
            config = create_from_profile(profile_name, "test", key_pair.private_key)
            profile = SECURITY_PROFILES[profile_name]
            
            # Verify configuration matches profile
            assert config.components.method == profile.components.method
            assert config.components.target_uri == profile.components.target_uri
            assert config.components.content_digest == profile.components.content_digest
            
        print(f"✓ All {len(SECURITY_PROFILES)} security profiles work correctly")
        
        
    except Exception as e:
        print(f"✗ Security profiles test failed: {e}")
        traceback.print_exc()
        assert False, f"Security profiles test failed: {e}"


def test_http_integration():
    """Test HTTP integration (if requests is available)"""
    print("\nTesting HTTP integration...")
    
    try:
        from datafold_sdk.signing.integration import create_signing_session
        from datafold_sdk.crypto.ed25519 import generate_key_pair
        from datafold_sdk.signing import create_from_profile
        
        key_pair = generate_key_pair()
        config = create_from_profile("standard", "test", key_pair.private_key)
        
        # This should work even if requests is not available
        session = create_signing_session(config)
        print("✓ Signing session creation")
        
        
        
    except ImportError as e:
        print(f"ℹ HTTP integration skipped (requests not available): {e}")
        
    except Exception as e:
        print(f"✗ HTTP integration test failed: {e}")
        traceback.print_exc()
        assert False, f"HTTP integration test failed: {e}"


def main():
    """Run all tests"""
    print("DataFold Python SDK - Request Signing Implementation Test")
    print("=" * 60)
    
    tests = [
        test_basic_imports,
        test_basic_functionality,
        test_utility_functions,
        test_security_profiles,
        test_http_integration,
    ]
    
    passed = 0
    total = len(tests)
    
    for test_func in tests:
        try:
            if test_func():
                passed += 1
        except Exception as e:
            print(f"✗ Test {test_func.__name__} crashed: {e}")
            traceback.print_exc()
    
    print("\n" + "=" * 60)
    print(f"Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! Implementation is working correctly.")
        return 0
    else:
        print("❌ Some tests failed. Check the output above for details.")
        return 1


if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)