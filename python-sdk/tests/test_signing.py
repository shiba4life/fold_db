"""
Test suite for RFC 9421 request signing functionality

This module tests the complete request signing implementation including
configuration, signing, canonical message construction, and HTTP integration.
"""

import pytest
import time
import uuid
from unittest.mock import Mock, patch

from datafold_sdk.signing import (
    # Core signing
    RFC9421Signer,
    create_signer,
    sign_request,
    # Types
    SignableRequest,
    SigningConfig,
    SignatureComponents,
    SigningOptions,
    SigningError,
    HttpMethod,
    SignatureAlgorithm,
    DigestAlgorithm,
    # Configuration
    create_signing_config,
    create_from_profile,
    SECURITY_PROFILES,
    DEFAULT_SIGNATURE_COMPONENTS,
    # Utilities
    generate_nonce,
    generate_timestamp,
    calculate_content_digest,
    validate_nonce,
    validate_timestamp,
    validate_signing_private_key,
    parse_url,
    # HTTP Integration
    create_signing_session,
    SigningSession,
)
from datafold_sdk.crypto.ed25519 import generate_key_pair
from datafold_sdk.exceptions import ValidationError


class TestSigningUtilities:
    """Test utility functions"""
    
    def test_generate_nonce(self):
        """Test nonce generation"""
        nonce = generate_nonce()
        assert isinstance(nonce, str)
        assert validate_nonce(nonce)
        
        # Generate multiple nonces to ensure uniqueness
        nonces = [generate_nonce() for _ in range(10)]
        assert len(set(nonces)) == 10  # All unique
    
    def test_generate_timestamp(self):
        """Test timestamp generation"""
        timestamp = generate_timestamp()
        assert isinstance(timestamp, int)
        assert validate_timestamp(timestamp)
        
        # Should be close to current time
        current_time = int(time.time())
        assert abs(timestamp - current_time) < 2  # Within 2 seconds
    
    def test_validate_nonce(self):
        """Test nonce validation"""
        # Valid UUID v4
        valid_nonce = str(uuid.uuid4())
        assert validate_nonce(valid_nonce)
        
        # Invalid formats
        assert not validate_nonce("not-a-uuid")
        assert not validate_nonce("12345678-1234-1234-1234-123456789012")  # Wrong version
        assert not validate_nonce("")
        assert not validate_nonce(None)
    
    def test_validate_timestamp(self):
        """Test timestamp validation"""
        # Valid timestamps
        assert validate_timestamp(int(time.time()))
        assert validate_timestamp(1000000000)  # 2001
        assert validate_timestamp(4000000000)  # 2096
        
        # Invalid timestamps
        assert not validate_timestamp(0)
        assert not validate_timestamp(-1)
        assert not validate_timestamp(946684799)  # Before 2000
        assert not validate_timestamp(4102444801)  # After 2100
        assert not validate_timestamp(1.5)  # Float
        assert not validate_timestamp("1000000000")  # String
    
    def test_validate_signing_private_key(self):
        """Test private key validation"""
        # Valid key
        key_pair = generate_key_pair()
        assert validate_signing_private_key(key_pair.private_key)
        
        # Invalid keys
        assert not validate_signing_private_key(b"too_short")
        assert not validate_signing_private_key(b"x" * 33)  # Too long
        assert not validate_signing_private_key(b"\x00" * 32)  # All zeros
        assert not validate_signing_private_key("not_bytes")
        assert not validate_signing_private_key(None)
    
    def test_parse_url(self):
        """Test URL parsing"""
        url = "https://api.example.com/path?param=value"
        parsed = parse_url(url)
        
        assert parsed["origin"] == "https://api.example.com"
        assert parsed["pathname"] == "/path"
        assert parsed["search"] == "?param=value"
        assert parsed["target_uri"] == "/path?param=value"
        
        # URL without query
        url2 = "http://localhost:8080/api/test"
        parsed2 = parse_url(url2)
        assert parsed2["target_uri"] == "/api/test"
        
        # Invalid URLs
        with pytest.raises(SigningError):
            parse_url("not-a-url")
        
        with pytest.raises(SigningError):
            parse_url("file://local/path")  # No netloc
    
    def test_calculate_content_digest(self):
        """Test content digest calculation"""
        content = "Hello, World!"
        
        # SHA-256
        digest = calculate_content_digest(content, DigestAlgorithm.SHA256)
        assert digest.algorithm == DigestAlgorithm.SHA256
        assert digest.header_value.startswith("sha-256=:")
        assert digest.header_value.endswith(":")
        
        # SHA-512
        digest512 = calculate_content_digest(content, DigestAlgorithm.SHA512)
        assert digest512.algorithm == DigestAlgorithm.SHA512
        assert digest512.header_value.startswith("sha-512=:")
        
        # Empty content
        empty_digest = calculate_content_digest("", DigestAlgorithm.SHA256)
        assert empty_digest.digest  # Should have a digest for empty string
        
        # Bytes content
        bytes_content = b"binary data"
        bytes_digest = calculate_content_digest(bytes_content, DigestAlgorithm.SHA256)
        assert bytes_digest.algorithm == DigestAlgorithm.SHA256
        
        # None content
        none_digest = calculate_content_digest(None, DigestAlgorithm.SHA256)
        assert none_digest.algorithm == DigestAlgorithm.SHA256


class TestSigningConfiguration:
    """Test signing configuration and builders"""
    
    def test_signature_components(self):
        """Test signature components creation"""
        # Default components
        components = SignatureComponents()
        assert components.method is True
        assert components.target_uri is True
        assert components.content_digest is True
        assert components.headers == []
        
        # Custom components
        custom = SignatureComponents(
            method=False,
            headers=['content-type', 'authorization']
        )
        assert custom.method is False
        assert 'content-type' in custom.headers
        assert 'authorization' in custom.headers
    
    def test_signing_config_builder(self):
        """Test signing configuration builder"""
        key_pair = generate_key_pair()
        
        config = (create_signing_config()
                 .key_id("test-key")
                 .private_key(key_pair.private_key)
                 .add_header("content-type")
                 .add_header("authorization")
                 .content_digest(True)
                 .build())
        
        assert config.key_id == "test-key"
        assert config.private_key == key_pair.private_key
        assert config.algorithm == SignatureAlgorithm.ED25519
        assert 'content-type' in config.components.headers
        assert 'authorization' in config.components.headers
        assert config.components.content_digest is True
    
    def test_security_profiles(self):
        """Test security profile application"""
        key_pair = generate_key_pair()
        
        # Test each profile
        for profile_name in SECURITY_PROFILES:
            config = create_from_profile(
                profile_name,
                "test-key",
                key_pair.private_key
            )
            
            assert config.key_id == "test-key"
            assert config.private_key == key_pair.private_key
            
            profile = SECURITY_PROFILES[profile_name]
            assert config.components.method == profile.components.method
            assert config.components.target_uri == profile.components.target_uri
            assert config.components.content_digest == profile.components.content_digest
    
    def test_config_validation(self):
        """Test configuration validation"""
        key_pair = generate_key_pair()
        
        # Valid config
        config = create_signing_config().key_id("test").private_key(key_pair.private_key).build()
        assert config.key_id == "test"
        
        # Missing key ID
        with pytest.raises(SigningError):
            create_signing_config().private_key(key_pair.private_key).build()
        
        # Missing private key
        with pytest.raises(SigningError):
            create_signing_config().key_id("test").build()
        
        # Invalid private key
        with pytest.raises(SigningError):
            create_signing_config().key_id("test").private_key(b"invalid").build()


class TestCanonicalMessage:
    """Test canonical message construction"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.key_pair = generate_key_pair()
        self.config = (create_signing_config()
                      .key_id("test-key")
                      .private_key(self.key_pair.private_key)
                      .headers(['content-type', 'authorization'])
                      .content_digest(True)
                      .build())
        
        self.request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={
                "content-type": "application/json",
                "authorization": "Bearer test-token"
            },
            body='{"message": "test"}'
        )
    
    def test_canonical_message_construction(self):
        """Test canonical message building"""
        signer = RFC9421Signer(self.config)
        result = signer.sign_request(self.request)
        
        canonical = result.canonical_message
        lines = canonical.split('\n')
        
        # Should have @method, @target-uri, content-type, authorization, content-digest, @signature-params
        assert len(lines) == 6
        assert lines[0] == '"@method": POST'
        assert lines[1] == '"@target-uri": /test'
        assert lines[2] == '"content-type": application/json'
        assert lines[3] == '"authorization": Bearer test-token'
        assert lines[4].startswith('"content-digest": sha-256=:')
        assert lines[5].startswith('"@signature-params": ')
    
    def test_get_request_no_body(self):
        """Test GET request without body"""
        get_request = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/users?limit=10",
            headers={"accept": "application/json"}
        )
        
        # Use minimal profile for GET requests
        config = create_from_profile("minimal", "test-key", self.key_pair.private_key)
        
        signer = RFC9421Signer(config)
        result = signer.sign_request(get_request)
        
        canonical = result.canonical_message
        lines = canonical.split('\n')
        
        # Should have @method, @target-uri, @signature-params (no content-digest for GET)
        assert '"@method": GET' in canonical
        assert '"@target-uri": /users?limit=10' in canonical
        assert 'content-digest' not in canonical
    
    def test_custom_components(self):
        """Test custom signature components"""
        custom_config = (create_signing_config()
                        .key_id("test-key")
                        .private_key(self.key_pair.private_key)
                        .headers(['content-type', 'authorization', 'x-custom'])
                        .content_digest(False)
                        .build())
        
        request_with_headers = SignableRequest(
            method=HttpMethod.PUT,
            url="https://api.example.com/resource/123",
            headers={
                "content-type": "application/json",
                "authorization": "Bearer token123",
                "x-custom": "custom-value"
            },
            body='{"updated": true}'
        )
        
        signer = RFC9421Signer(custom_config)
        result = signer.sign_request(request_with_headers)
        
        canonical = result.canonical_message
        
        assert '"content-type": application/json' in canonical
        assert '"authorization": Bearer token123' in canonical
        assert '"x-custom": custom-value' in canonical
        assert 'content-digest' not in canonical  # Disabled


class TestRFC9421Signer:
    """Test the main RFC 9421 signer implementation"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.key_pair = generate_key_pair()
        self.config = create_from_profile("standard", "test-client", self.key_pair.private_key)
        self.signer = RFC9421Signer(self.config)
        
        self.request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.datafold.com/api/crypto/keys/register",
            headers={"content-type": "application/json"},
            body='{"client_id": "test", "public_key": "abc123"}'
        )
    
    def teardown_method(self):
        """Clean up test fixtures"""
        self.key_pair = None
        self.config = None
        self.signer = None
        self.request = None
    
    def test_sign_request_basic(self):
        """Test basic request signing"""
        result = self.signer.sign_request(self.request)
        
        assert isinstance(result.signature_input, str)
        assert isinstance(result.signature, str)
        assert isinstance(result.headers, dict)
        assert isinstance(result.canonical_message, str)
        
        # Check signature format
        assert result.signature.startswith('sig1=:')
        assert result.signature.endswith(':')
        
        # Check headers
        assert 'signature-input' in result.headers
        assert 'signature' in result.headers
        assert 'content-digest' in result.headers
    
    def test_sign_request_with_options(self):
        """Test signing with custom options"""
        custom_nonce = str(uuid.uuid4())
        custom_timestamp = int(time.time())
        
        options = SigningOptions(
            nonce=custom_nonce,
            timestamp=custom_timestamp,
            digest_algorithm=DigestAlgorithm.SHA512
        )
        
        result = self.signer.sign_request(self.request, options)
        
        # Check that custom values were used
        assert custom_nonce in result.signature_input
        assert str(custom_timestamp) in result.signature_input
        assert 'sha-512' in result.headers['content-digest']
    
    def test_sign_request_performance(self):
        """Test signing performance meets requirements (<10ms)"""
        start_time = time.perf_counter()
        
        # Sign multiple requests to get average
        for _ in range(10):
            self.signer.sign_request(self.request)
        
        end_time = time.perf_counter()
        avg_time_ms = ((end_time - start_time) / 10) * 1000
        
        # Should be well under 10ms per request
        assert avg_time_ms < 10, f"Signing took {avg_time_ms:.2f}ms (target: <10ms)"
    
    def test_different_http_methods(self):
        """Test signing different HTTP methods"""
        methods = [HttpMethod.GET, HttpMethod.POST, HttpMethod.PUT, HttpMethod.DELETE, HttpMethod.PATCH]
        
        for method in methods:
            # Provide content-type header for requests with body, use minimal profile for GET
            if method == HttpMethod.GET:
                # Use minimal profile for GET requests without body
                minimal_config = create_from_profile("minimal", "test-key", self.key_pair.private_key)
                signer = RFC9421Signer(minimal_config)
                request = SignableRequest(
                    method=method,
                    url="https://api.example.com/test",
                    headers={"accept": "application/json"},
                    body=None
                )
            else:
                # Use standard profile but provide required content-type header
                signer = self.signer
                request = SignableRequest(
                    method=method,
                    url="https://api.example.com/test",
                    headers={"content-type": "application/json", "accept": "application/json"},
                    body='{"test": true}'
                )
            
            result = signer.sign_request(request)
            
            assert f'"@method": {method.value}' in result.canonical_message
            assert 'sig1=:' in result.signature
    
    def test_url_components(self):
        """Test various URL formats"""
        # Use minimal profile to avoid header requirements
        minimal_config = create_from_profile("minimal", "test-key", self.key_pair.private_key)
        minimal_signer = RFC9421Signer(minimal_config)
        
        urls = [
            "https://api.example.com/simple",
            "https://api.example.com/path/with/segments",
            "https://api.example.com/query?param=value&other=123",
            "https://api.example.com/both/path?and=query",
            "http://localhost:8080/local/api",
        ]
        
        for url in urls:
            request = SignableRequest(
                method=HttpMethod.GET,
                url=url,
                headers={}
            )
            
            result = minimal_signer.sign_request(request)
            
            # Parse expected target-uri
            from urllib.parse import urlparse
            parsed = urlparse(url)
            expected_target = parsed.path + ('?' + parsed.query if parsed.query else '')
            if not expected_target:
                expected_target = "/"
            
            assert f'"@target-uri": {expected_target}' in result.canonical_message
    
    def test_error_handling(self):
        """Test error handling for invalid requests"""
        # Invalid URL
        with pytest.raises(SigningError):
            bad_request = SignableRequest(
                method=HttpMethod.GET,
                url="not-a-valid-url",
                headers={}
            )
            self.signer.sign_request(bad_request)
        
        # Missing required header
        config_with_required_header = (create_signing_config()
                                     .key_id("test")
                                     .private_key(self.key_pair.private_key)
                                     .headers(['authorization'])
                                     .build())
        
        signer_with_required = RFC9421Signer(config_with_required_header)
        
        with pytest.raises(SigningError):
            signer_with_required.sign_request(self.request)  # Missing authorization header


class TestHTTPIntegration:
    """Test HTTP client integration"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.key_pair = generate_key_pair()
        self.config = create_from_profile("standard", "test-client", self.key_pair.private_key)
    
    def teardown_method(self):
        """Clean up test fixtures"""
        self.key_pair = None
        self.config = None
    
    @pytest.mark.skipif(not hasattr(create_signing_session, '__call__'), 
                       reason="HTTP integration requires requests")
    def test_signing_session_creation(self):
        """Test signing session creation"""
        session = create_signing_session(self.config)
        
        assert session.signing_config == self.config
        assert session.auto_sign is True
        assert hasattr(session, 'get')
        assert hasattr(session, 'post')
    
    @pytest.mark.skipif(not hasattr(create_signing_session, '__call__'), 
                       reason="HTTP integration requires requests")
    def test_signing_session_configure(self):
        """Test signing session configuration"""
        session = create_signing_session()
        assert session.signing_config is None
        
        session.configure_signing(self.config)
        assert session.signing_config == self.config
        assert session.auto_sign is True
        
        session.disable_signing()
        assert session.auto_sign is False
    
    @patch('requests.Session.request')
    def test_automatic_signing(self, mock_request):
        """Test automatic request signing"""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"success": True}
        mock_request.return_value = mock_response
        
        session = create_signing_session(self.config)
        
        # Make a request
        response = session.post(
            "https://api.example.com/test",
            json={"data": "test"}
        )
        
        # Verify request was called
        assert mock_request.called
        call_args = mock_request.call_args
        
        # Check that signature headers were added
        headers = call_args[1]['headers']
        assert 'signature-input' in headers
        assert 'signature' in headers
        assert 'content-digest' in headers


class TestEndToEndIntegration:
    """End-to-end integration tests"""
    
    def test_complete_workflow(self):
        """Test complete signing workflow"""
        # 1. Generate key pair
        key_pair = generate_key_pair()
        
        # 2. Create signing configuration
        config = (create_signing_config()
                 .key_id("e2e-test-client")
                 .private_key(key_pair.private_key)
                 .profile("standard")
                 .build())
        
        # 3. Create request
        request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.datafold.com/api/crypto/keys/register",
            headers={
                "content-type": "application/json",
                "user-agent": "DataFold-Python-SDK/1.0.0"
            },
            body='{"client_id": "e2e-test", "public_key": "' + key_pair.public_key.hex() + '"}'
        )
        
        # 4. Sign request
        signer = RFC9421Signer(config)
        result = signer.sign_request(request)
        
        # 5. Verify result structure
        assert all(key in result.headers for key in ['signature-input', 'signature', 'content-digest'])
        
        # 6. Verify signature format compliance
        assert result.signature.startswith('sig1=:')
        assert result.signature.endswith(':')
        
        # 7. Verify signature input format
        assert 'created=' in result.signature_input
        assert 'keyid="e2e-test-client"' in result.signature_input
        assert 'alg="ed25519"' in result.signature_input
        assert 'nonce=' in result.signature_input
        
        # 8. Verify canonical message structure
        lines = result.canonical_message.split('\n')
        assert len(lines) >= 3  # At least method, target-uri, signature-params
        assert lines[-1].startswith('"@signature-params": ')
    
    def test_compatibility_check(self):
        """Test SDK compatibility verification"""
        from datafold_sdk.signing.rfc9421_signer import verify_signer_compatibility
        
        # Should be compatible with cryptography installed
        assert verify_signer_compatibility() is True
    
    def test_real_world_scenarios(self):
        """Test real-world usage scenarios"""
        key_pair = generate_key_pair()
        
        # Scenario 1: API key registration with all required headers for strict profile
        register_config = create_from_profile("strict", "new-client", key_pair.private_key)
        body = '{"client_id": "new-client", "public_key": "' + key_pair.public_key.hex() + '"}'
        register_request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.datafold.com/api/crypto/keys/register",
            headers={
                "content-type": "application/json",
                "content-length": str(len(body.encode('utf-8'))),
                "user-agent": "MyApp/1.0",
                "authorization": "Bearer test-token"
            },
            body=body
        )
        
        signer = RFC9421Signer(register_config)
        result = signer.sign_request(register_request)
        assert result.signature
        
        # Scenario 2: Data query with minimal signing
        query_config = create_from_profile("minimal", "query-client", key_pair.private_key)
        query_request = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.datafold.com/api/data/query?table=users&limit=100",
            headers={"accept": "application/json"}
        )
        
        query_signer = RFC9421Signer(query_config)
        query_result = query_signer.sign_request(query_request)
        assert query_result.signature
        
        # Different signatures for different requests
        assert result.signature != query_result.signature


if __name__ == "__main__":
    pytest.main([__file__, "-v"])