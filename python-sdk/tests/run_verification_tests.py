"""
Test runner for signature verification functionality

This script runs comprehensive tests for the verification module to ensure
all components work correctly together.
"""

import asyncio
import sys
import time
from typing import Dict, Any

# Add parent directory to path for imports
sys.path.insert(0, '../src')

from datafold_sdk.verification import (
    VerificationConfig,
    RFC9421Verifier,
    RFC9421Inspector,
    VerifiableResponse,
    create_verifier,
    VERIFICATION_POLICIES,
)
from datafold_sdk.verification.utils import (
    extract_signature_data,
    validate_signature_format,
)
from datafold_sdk.verification.middleware import (
    ResponseVerificationConfig,
    create_response_verification_middleware,
    create_batch_verifier,
)
from datafold_sdk.signing.types import SignableRequest, HttpMethod
from datafold_sdk.crypto.ed25519 import generate_key_pair


def test_module_imports():
    """Test that all modules can be imported correctly"""
    print("Testing module imports...")
    
    try:
        from datafold_sdk.verification import (
            VerificationStatus, VerificationPolicy, VerificationConfig,
            RFC9421Verifier, RFC9421Inspector, VerificationResult
        )
        print("✓ Core verification modules imported successfully")
        
        from datafold_sdk.verification.policies import (
            STRICT_VERIFICATION_POLICY, STANDARD_VERIFICATION_POLICY
        )
        print("✓ Verification policies imported successfully")
        
        from datafold_sdk.verification.middleware import (
            ResponseVerificationMiddleware, BatchVerifier
        )
        print("✓ Middleware components imported successfully")
        
        return True
    except ImportError as e:
        print(f"✗ Import failed: {e}")
        return False


def test_configuration_creation():
    """Test verification configuration creation"""
    print("\nTesting configuration creation...")
    
    try:
        # Generate test keys
        key_pair = generate_key_pair()
        
        # Create configuration
        config = VerificationConfig(
            default_policy='standard',
            public_keys={'test-key': key_pair.public_key}
        )
        
        assert config.default_policy == 'standard'
        assert 'test-key' in config.public_keys
        assert len(config.public_keys['test-key']) == 32
        
        print("✓ VerificationConfig created successfully")
        return True
    except Exception as e:
        print(f"✗ Configuration creation failed: {e}")
        return False


def test_policies():
    """Test verification policies"""
    print("\nTesting verification policies...")
    
    try:
        # Test built-in policies exist
        assert 'strict' in VERIFICATION_POLICIES
        assert 'standard' in VERIFICATION_POLICIES
        assert 'lenient' in VERIFICATION_POLICIES
        assert 'legacy' in VERIFICATION_POLICIES
        
        # Test policy properties
        strict_policy = VERIFICATION_POLICIES['strict']
        assert strict_policy.verify_timestamp is True
        assert strict_policy.verify_nonce is True
        assert '@method' in strict_policy.required_components
        
        print("✓ Verification policies work correctly")
        return True
    except Exception as e:
        print(f"✗ Policy test failed: {e}")
        return False


def test_signature_format_validation():
    """Test signature format validation"""
    print("\nTesting signature format validation...")
    
    try:
        # Valid headers
        valid_headers = {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:'
        }
        
        is_valid, issues = validate_signature_format(valid_headers)
        assert is_valid is True
        
        # Invalid headers (missing signature)
        invalid_headers = {
            'signature-input': 'sig1=("@method");created=1640995200;keyid="test";alg="ed25519";nonce="nonce"'
        }
        
        is_valid, issues = validate_signature_format(invalid_headers)
        assert is_valid is False
        assert len(issues) > 0
        
        print("✓ Signature format validation works correctly")
        return True
    except Exception as e:
        print(f"✗ Signature format validation failed: {e}")
        return False


def test_signature_data_extraction():
    """Test signature data extraction"""
    print("\nTesting signature data extraction...")
    
    try:
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri" "content-type");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:',
            'content-type': 'application/json'
        }
        
        signature_data = extract_signature_data(headers)
        
        assert signature_data.signature_id == 'sig1'
        assert '@method' in signature_data.covered_components
        assert '@target-uri' in signature_data.covered_components
        assert 'content-type' in signature_data.covered_components
        assert signature_data.params.keyid == 'test-key'
        
        print("✓ Signature data extraction works correctly")
        return True
    except Exception as e:
        print(f"✗ Signature data extraction failed: {e}")
        return False


async def test_verifier_creation():
    """Test verifier creation and basic operations"""
    print("\nTesting verifier creation...")
    
    try:
        # Generate test keys
        key_pair = generate_key_pair()
        
        # Create configuration
        config = VerificationConfig(
            public_keys={'test-key': key_pair.public_key}
        )
        
        # Create verifier
        verifier = create_verifier(config)
        assert isinstance(verifier, RFC9421Verifier)
        
        # Test key management
        new_key_pair = generate_key_pair()
        verifier.add_public_key('new-key', new_key_pair.public_key)
        assert 'new-key' in verifier.config.public_keys
        
        verifier.remove_public_key('new-key')
        assert 'new-key' not in verifier.config.public_keys
        
        print("✓ Verifier creation and key management work correctly")
        return True
    except Exception as e:
        print(f"✗ Verifier creation failed: {e}")
        return False


def test_inspector():
    """Test signature inspector"""
    print("\nTesting signature inspector...")
    
    try:
        inspector = RFC9421Inspector()
        
        # Test format inspection
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:'
        }
        
        analysis = inspector.inspect_format(headers)
        assert analysis.is_valid_rfc9421 is True
        assert 'sig1' in analysis.signature_ids
        
        # Test component analysis
        signature_data = extract_signature_data(headers)
        component_analysis = inspector.analyze_components(signature_data)
        assert '@method' in component_analysis.valid_components
        assert component_analysis.security_assessment.level in ['low', 'medium', 'high']
        
        print("✓ Signature inspector works correctly")
        return True
    except Exception as e:
        print(f"✗ Inspector test failed: {e}")
        return False


async def test_middleware():
    """Test verification middleware"""
    print("\nTesting verification middleware...")
    
    try:
        # Generate test keys
        key_pair = generate_key_pair()
        
        # Create configuration
        verification_config = VerificationConfig(
            public_keys={'test-key': key_pair.public_key}
        )
        
        # Create response middleware
        response_config = ResponseVerificationConfig(
            verification_config=verification_config,
            default_policy='lenient'
        )
        
        middleware = create_response_verification_middleware(response_config)
        assert isinstance(middleware, ResponseVerificationMiddleware)
        
        # Create batch verifier
        batch_verifier = create_batch_verifier(verification_config)
        
        # Test empty batch
        results = await batch_verifier.verify_batch([])
        assert len(results) == 0
        
        stats = batch_verifier.get_batch_stats([])
        assert stats['total'] == 0
        
        print("✓ Verification middleware works correctly")
        return True
    except Exception as e:
        print(f"✗ Middleware test failed: {e}")
        return False


async def test_integration():
    """Test full integration scenario"""
    print("\nTesting integration scenario...")
    
    try:
        # Generate test keys
        key_pair = generate_key_pair()
        
        # Create configuration
        config = VerificationConfig(
            default_policy='lenient',  # Use lenient for easier testing
            public_keys={'test-key': key_pair.public_key}
        )
        
        verifier = create_verifier(config)
        
        # Create test request
        request = SignableRequest(
            method=HttpMethod.GET,
            url='https://example.com/test',
            headers={'host': 'example.com'},
            body=None
        )
        
        # Create signature headers (mock)
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:'
        }
        
        # Mock the cryptographic verification
        from unittest.mock import patch
        with patch('datafold_sdk.crypto.ed25519.verify_signature', return_value=True):
            result = await verifier.verify(request, headers)
        
        assert result is not None
        assert hasattr(result, 'status')
        assert hasattr(result, 'signature_valid')
        assert hasattr(result, 'checks')
        
        print("✓ Integration scenario works correctly")
        return True
    except Exception as e:
        print(f"✗ Integration test failed: {e}")
        return False


def test_error_handling():
    """Test error handling"""
    print("\nTesting error handling...")
    
    try:
        from datafold_sdk.verification import VerificationError, VerificationResult
        from datafold_sdk.verification.types import VerificationStatus
        
        # Test error creation
        error = VerificationError(
            'Test error',
            'TEST_ERROR',
            {'detail': 'test'}
        )
        
        assert error.message == 'Test error'
        assert error.code == 'TEST_ERROR'
        assert 'TEST_ERROR' in str(error)
        
        # Test error result creation
        result = VerificationResult.create_error(
            'TEST_ERROR',
            'Test error message'
        )
        
        assert result.status == VerificationStatus.ERROR
        assert result.signature_valid is False
        assert result.error['code'] == 'TEST_ERROR'
        
        print("✓ Error handling works correctly")
        return True
    except Exception as e:
        print(f"✗ Error handling test failed: {e}")
        return False


async def run_all_tests():
    """Run all verification tests"""
    print("DataFold Python SDK Verification Tests")
    print("=" * 50)
    
    test_results = []
    
    # Run synchronous tests
    sync_tests = [
        test_module_imports,
        test_configuration_creation,
        test_policies,
        test_signature_format_validation,
        test_signature_data_extraction,
        test_inspector,
        test_error_handling,
    ]
    
    for test in sync_tests:
        try:
            result = test()
            test_results.append(result)
        except Exception as e:
            print(f"✗ Test {test.__name__} failed with exception: {e}")
            test_results.append(False)
    
    # Run asynchronous tests
    async_tests = [
        test_verifier_creation,
        test_middleware,
        test_integration,
    ]
    
    for test in async_tests:
        try:
            result = await test()
            test_results.append(result)
        except Exception as e:
            print(f"✗ Test {test.__name__} failed with exception: {e}")
            test_results.append(False)
    
    # Summary
    passed = sum(test_results)
    total = len(test_results)
    
    print("\n" + "=" * 50)
    print(f"Test Results: {passed}/{total} passed")
    
    if passed == total:
        print("🎉 All tests passed!")
        return True
    else:
        print(f"❌ {total - passed} tests failed")
        return False


if __name__ == '__main__':
    success = asyncio.run(run_all_tests())
    sys.exit(0 if success else 1)