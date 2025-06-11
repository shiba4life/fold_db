"""
Unit tests for signature verification functionality

This module tests the RFC 9421 HTTP Message Signatures verification implementation
including verifier, policies, inspector, and middleware components.
"""

import pytest
import asyncio
import time
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any

from datafold_sdk.verification import (
    RFC9421Verifier,
    VerificationConfig,
    VerificationPolicy,
    VerificationResult,
    VerificationStatus,
    VerificationError,
    ExtractedSignatureData,
    VerifiableResponse,
    RFC9421Inspector,
    create_verifier,
    verify_signature,
    VERIFICATION_POLICIES,
    STRICT_VERIFICATION_POLICY,
    STANDARD_VERIFICATION_POLICY,
)
from datafold_sdk.verification.utils import (
    extract_signature_data,
    validate_signature_format,
    reconstruct_canonical_message,
    verify_content_digest,
)
from datafold_sdk.verification.middleware import (
    ResponseVerificationMiddleware,
    RequestVerificationMiddleware,
    BatchVerifier,
)
from datafold_sdk.signing.types import (
    SignableRequest,
    SignatureParams,
    SignatureAlgorithm,
    HttpMethod,
)
from datafold_sdk.crypto.ed25519 import generate_key_pair


class TestVerificationConfig:
    """Test verification configuration"""
    
    def test_create_config(self):
        """Test creating verification configuration"""
        config = VerificationConfig()
        
        assert config.default_policy == "standard"
        assert isinstance(config.policies, dict)
        assert isinstance(config.public_keys, dict)
        assert config.trusted_key_sources == []
        assert config.performance_monitoring['enabled'] is True
    
    def test_config_with_keys(self):
        """Test configuration with public keys"""
        key_pair = generate_key_pair()
        
        config = VerificationConfig(
            public_keys={'test-key': key_pair.public_key}
        )
        
        assert 'test-key' in config.public_keys
        assert config.public_keys['test-key'] == key_pair.public_key


class TestVerificationPolicies:
    """Test verification policies"""
    
    def test_builtin_policies_exist(self):
        """Test that built-in policies exist"""
        assert 'strict' in VERIFICATION_POLICIES
        assert 'standard' in VERIFICATION_POLICIES
        assert 'lenient' in VERIFICATION_POLICIES
        assert 'legacy' in VERIFICATION_POLICIES
    
    def test_strict_policy_properties(self):
        """Test strict policy properties"""
        policy = STRICT_VERIFICATION_POLICY
        
        assert policy.name == 'strict'
        assert policy.verify_timestamp is True
        assert policy.max_timestamp_age == 300
        assert policy.verify_nonce is True
        assert policy.verify_content_digest is True
        assert '@method' in policy.required_components
        assert '@target-uri' in policy.required_components
        assert 'ed25519' in policy.allowed_algorithms
        assert len(policy.custom_rules) > 0
    
    def test_standard_policy_properties(self):
        """Test standard policy properties"""
        policy = STANDARD_VERIFICATION_POLICY
        
        assert policy.name == 'standard'
        assert policy.verify_timestamp is True
        assert policy.max_timestamp_age == 900
        assert policy.verify_nonce is True
        assert policy.verify_content_digest is True
        assert policy.require_all_headers is False


class TestExtractSignatureData:
    """Test signature data extraction"""
    
    def test_extract_valid_signature_data(self):
        """Test extracting valid signature data"""
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri" "content-type");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce-uuid"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:',
            'content-type': 'application/json'
        }
        
        signature_data = extract_signature_data(headers)
        
        assert signature_data.signature_id == 'sig1'
        assert signature_data.signature == 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
        assert '@method' in signature_data.covered_components
        assert '@target-uri' in signature_data.covered_components
        assert 'content-type' in signature_data.covered_components
        assert signature_data.params.keyid == 'test-key'
        assert signature_data.params.alg == SignatureAlgorithm.ED25519
    
    def test_extract_missing_signature_input(self):
        """Test extraction with missing signature-input header"""
        headers = {
            'signature': 'sig1=:abcdef:'
        }
        
        with pytest.raises(VerificationError) as exc_info:
            extract_signature_data(headers)
        
        assert 'MISSING_SIGNATURE_INPUT' in str(exc_info.value)
    
    def test_extract_missing_signature(self):
        """Test extraction with missing signature header"""
        headers = {
            'signature-input': 'sig1=("@method");created=1640995200;keyid="test";alg="ed25519";nonce="nonce"'
        }
        
        with pytest.raises(VerificationError) as exc_info:
            extract_signature_data(headers)
        
        assert 'MISSING_SIGNATURE' in str(exc_info.value)


class TestSignatureFormatValidation:
    """Test signature format validation"""
    
    def test_valid_signature_format(self):
        """Test valid signature format"""
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:'
        }
        
        is_valid, issues = validate_signature_format(headers)
        
        assert is_valid is True
        assert len([issue for issue in issues if issue.severity == 'error']) == 0
    
    def test_invalid_signature_length(self):
        """Test invalid signature length"""
        headers = {
            'signature-input': 'sig1=("@method");created=1640995200;keyid="test";alg="ed25519";nonce="nonce"',
            'signature': 'sig1=:abcdef:'  # Too short
        }
        
        is_valid, issues = validate_signature_format(headers)
        
        # Should have warning about signature length
        warnings = [issue for issue in issues if issue.severity == 'warning' and 'length' in issue.message.lower()]
        assert len(warnings) > 0


class TestRFC9421Verifier:
    """Test the main RFC 9421 verifier"""
    
    @pytest.fixture
    def key_pair(self):
        """Generate test key pair"""
        return generate_key_pair()
    
    @pytest.fixture
    def config(self, key_pair):
        """Create test verification config"""
        return VerificationConfig(
            public_keys={'test-key': key_pair.public_key}
        )
    
    @pytest.fixture
    def verifier(self, config):
        """Create test verifier"""
        return RFC9421Verifier(config)
    
    def test_verifier_creation(self, config):
        """Test verifier creation"""
        verifier = RFC9421Verifier(config)
        assert verifier.config.default_policy == 'standard'
    
    def test_add_public_key(self, verifier, key_pair):
        """Test adding public key"""
        verifier.add_public_key('new-key', key_pair.public_key)
        assert 'new-key' in verifier.config.public_keys
    
    def test_remove_public_key(self, verifier):
        """Test removing public key"""
        verifier.remove_public_key('test-key')
        assert 'test-key' not in verifier.config.public_keys
    
    def test_get_policy_default(self, verifier):
        """Test getting default policy"""
        policy = verifier._get_policy(None)
        assert policy.name == 'standard'
    
    def test_get_policy_by_name(self, verifier):
        """Test getting policy by name"""
        policy = verifier._get_policy('strict')
        assert policy.name == 'strict'
    
    def test_get_unknown_policy(self, verifier):
        """Test getting unknown policy"""
        with pytest.raises(VerificationError) as exc_info:
            verifier._get_policy('nonexistent')
        
        assert 'UNKNOWN_POLICY' in str(exc_info.value)
    
    @pytest.mark.asyncio
    async def test_get_public_key_from_config(self, verifier, key_pair):
        """Test getting public key from configuration"""
        key = await verifier._get_public_key('test-key', None, False)
        assert key == key_pair.public_key
    
    @pytest.mark.asyncio
    async def test_get_public_key_provided(self, verifier, key_pair):
        """Test getting provided public key"""
        other_key_pair = generate_key_pair()
        key = await verifier._get_public_key('test-key', other_key_pair.public_key, False)
        assert key == other_key_pair.public_key
    
    @pytest.mark.asyncio
    async def test_get_public_key_not_found(self, verifier):
        """Test getting public key that doesn't exist"""
        with pytest.raises(VerificationError) as exc_info:
            await verifier._get_public_key('nonexistent', None, True)
        
        assert 'PUBLIC_KEY_NOT_FOUND' in str(exc_info.value)


class TestContentDigestVerification:
    """Test content digest verification"""
    
    def test_verify_valid_sha256_digest(self):
        """Test verifying valid SHA-256 digest"""
        content = "Hello, world!"
        digest_info = {
            'algorithm': 'sha-256',
            'value': 'MV9b23bQeMQ7isAGTkoBZGErH853yGk0W/yUx1iU7dM='  # SHA-256 of "Hello, world!"
        }
        
        result = verify_content_digest(content, digest_info)
        assert result is True
    
    def test_verify_invalid_digest(self):
        """Test verifying invalid digest"""
        content = "Hello, world!"
        digest_info = {
            'algorithm': 'sha-256',
            'value': 'invalid-digest-value='
        }
        
        result = verify_content_digest(content, digest_info)
        assert result is False
    
    def test_verify_unsupported_algorithm(self):
        """Test verifying with unsupported algorithm"""
        content = "Hello, world!"
        digest_info = {
            'algorithm': 'md5',
            'value': 'some-digest='
        }
        
        result = verify_content_digest(content, digest_info)
        assert result is False


class TestRFC9421Inspector:
    """Test signature inspector"""
    
    @pytest.fixture
    def inspector(self):
        """Create test inspector"""
        return RFC9421Inspector()
    
    def test_inspect_valid_format(self, inspector):
        """Test inspecting valid signature format"""
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:'
        }
        
        analysis = inspector.inspect_format(headers)
        
        assert analysis.is_valid_rfc9421 is True
        assert 'sig1' in analysis.signature_ids
        assert 'signature-input' in analysis.signature_headers
        assert 'signature' in analysis.signature_headers
    
    def test_inspect_missing_headers(self, inspector):
        """Test inspecting format with missing headers"""
        headers = {}
        
        analysis = inspector.inspect_format(headers)
        
        assert analysis.is_valid_rfc9421 is False
        error_issues = [issue for issue in analysis.issues if issue.severity == 'error']
        assert len(error_issues) >= 2  # Missing signature-input and signature
    
    def test_analyze_components(self, inspector):
        """Test analyzing signature components"""
        signature_data = ExtractedSignatureData(
            signature_id='sig1',
            signature='abcdef',
            covered_components=['@method', '@target-uri', 'content-type'],
            params=SignatureParams(
                created=int(time.time()),
                keyid='test-key',
                alg=SignatureAlgorithm.ED25519,
                nonce='test-nonce'
            )
        )
        
        analysis = inspector.analyze_components(signature_data)
        
        assert '@method' in analysis.valid_components
        assert '@target-uri' in analysis.valid_components
        assert 'content-type' in analysis.valid_components
        assert analysis.security_assessment.level in ['low', 'medium', 'high']
    
    def test_validate_parameters(self, inspector):
        """Test validating signature parameters"""
        params = SignatureParams(
            created=int(time.time()),
            keyid='test-key',
            alg=SignatureAlgorithm.ED25519,
            nonce='550e8400-e29b-41d4-a716-446655440000'  # Valid UUID
        )
        
        validation = inspector.validate_parameters(params)
        
        assert validation.all_valid is True
        assert validation.parameters['created']['valid'] is True
        assert validation.parameters['keyid']['valid'] is True
        assert validation.parameters['alg']['valid'] is True
        assert validation.parameters['nonce']['valid'] is True


class TestVerificationMiddleware:
    """Test verification middleware"""
    
    @pytest.fixture
    def key_pair(self):
        """Generate test key pair"""
        return generate_key_pair()
    
    @pytest.fixture
    def config(self, key_pair):
        """Create test verification config"""
        return VerificationConfig(
            public_keys={'test-key': key_pair.public_key}
        )
    
    @pytest.fixture
    def response_middleware_config(self, config):
        """Create response middleware config"""
        from datafold_sdk.verification.middleware import ResponseVerificationConfig
        return ResponseVerificationConfig(
            verification_config=config,
            default_policy='standard'
        )
    
    @pytest.fixture
    def request_middleware_config(self, config):
        """Create request middleware config"""
        from datafold_sdk.verification.middleware import RequestVerificationConfig
        return RequestVerificationConfig(
            verification_config=config,
            default_policy='standard'
        )
    
    def test_response_middleware_creation(self, response_middleware_config):
        """Test creating response middleware"""
        middleware = ResponseVerificationMiddleware(response_middleware_config)
        assert middleware.config == response_middleware_config
        assert isinstance(middleware.verifier, RFC9421Verifier)
    
    def test_request_middleware_creation(self, request_middleware_config):
        """Test creating request middleware"""
        middleware = RequestVerificationMiddleware(request_middleware_config)
        assert middleware.config == request_middleware_config
        assert isinstance(middleware.verifier, RFC9421Verifier)


class TestBatchVerifier:
    """Test batch verification"""
    
    @pytest.fixture
    def key_pair(self):
        """Generate test key pair"""
        return generate_key_pair()
    
    @pytest.fixture
    def config(self, key_pair):
        """Create test verification config"""
        return VerificationConfig(
            public_keys={'test-key': key_pair.public_key}
        )
    
    @pytest.fixture
    def batch_verifier(self, config):
        """Create batch verifier"""
        return BatchVerifier(config)
    
    @pytest.mark.asyncio
    async def test_verify_empty_batch(self, batch_verifier):
        """Test verifying empty batch"""
        results = await batch_verifier.verify_batch([])
        assert len(results) == 0
    
    def test_get_batch_stats_empty(self, batch_verifier):
        """Test getting stats for empty batch"""
        stats = batch_verifier.get_batch_stats([])
        
        assert stats['total'] == 0
        assert stats['valid'] == 0
        assert stats['invalid'] == 0
        assert stats['errors'] == 0
        assert stats['success_rate'] == 0
        assert stats['average_time'] == 0


class TestVerificationIntegration:
    """Integration tests for verification functionality"""
    
    @pytest.fixture
    def key_pair(self):
        """Generate test key pair"""
        return generate_key_pair()
    
    @pytest.fixture
    def config(self, key_pair):
        """Create test verification config"""
        return VerificationConfig(
            default_policy='lenient',  # Use lenient for easier testing
            public_keys={'test-key': key_pair.public_key}
        )
    
    @pytest.mark.asyncio
    async def test_verify_signature_convenience_function(self, config, key_pair):
        """Test the convenience verify_signature function"""
        # Create a mock request
        request = SignableRequest(
            method=HttpMethod.GET,
            url='https://example.com/test',
            headers={'host': 'example.com'},
            body=None
        )
        
        # Create mock signature headers (would normally come from signing)
        headers = {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key";alg="ed25519";nonce="test-nonce"',
            'signature': 'sig1=:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:'
        }
        
        # Mock the cryptographic verification to return True
        with patch('datafold_sdk.crypto.ed25519.verify_signature', return_value=True):
            result = await verify_signature(request, headers, config, 'lenient')
        
        # Should pass format validation even if crypto fails
        assert isinstance(result, VerificationResult)
        assert result.status in [VerificationStatus.VALID, VerificationStatus.INVALID]
    
    def test_create_verifier_function(self, config):
        """Test the create_verifier convenience function"""
        verifier = create_verifier(config)
        
        assert isinstance(verifier, RFC9421Verifier)
        assert verifier.config.default_policy == 'lenient'


class TestErrorHandling:
    """Test error handling in verification"""
    
    def test_verification_error_creation(self):
        """Test creating verification error"""
        error = VerificationError(
            'Test error message',
            'TEST_ERROR_CODE',
            {'detail': 'test detail'}
        )
        
        assert error.message == 'Test error message'
        assert error.code == 'TEST_ERROR_CODE'
        assert error.details['detail'] == 'test detail'
        assert 'TEST_ERROR_CODE' in str(error)
    
    def test_verification_result_create_error(self):
        """Test creating error verification result"""
        result = VerificationResult.create_error(
            'TEST_ERROR',
            'Test error message',
            {'detail': 'error detail'}
        )
        
        assert result.status == VerificationStatus.ERROR
        assert result.signature_valid is False
        assert result.error['code'] == 'TEST_ERROR'
        assert result.error['message'] == 'Test error message'
        assert result.error['details']['detail'] == 'error detail'


if __name__ == '__main__':
    pytest.main([__file__])