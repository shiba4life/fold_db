"""
Integration tests for DataFold SDK server communication

These tests validate end-to-end workflows including key generation,
server registration, and signature verification.
"""

import pytest
import os
import tempfile
import time
from unittest.mock import Mock, patch, MagicMock
from typing import Dict, Any

# Import SDK modules
from datafold_sdk import (
    DataFoldClient, ClientSession, quick_setup,
    generate_key_pair, sign_message,
    DataFoldHttpClient, create_client, ServerConfig,
    PublicKeyRegistration, SignatureVerificationResult,
    ServerCommunicationError, ValidationError
)
from datafold_sdk.http_client import DataFoldHttpClient
from datafold_sdk.integration import DataFoldClient


# Test fixtures and mocks

@pytest.fixture
def mock_requests():
    """Mock requests module for testing without actual network calls."""
    with patch('datafold_sdk.http_client.requests') as mock_requests:
        # Mock session
        mock_session = MagicMock()
        mock_requests.Session.return_value = mock_session
        
        # Mock successful response
        mock_response = MagicMock()
        mock_response.ok = True
        mock_response.status_code = 200
        mock_session.request.return_value = mock_response
        
        yield mock_requests, mock_session, mock_response


@pytest.fixture
def server_config():
    """Test server configuration."""
    return ServerConfig(
        base_url="http://localhost:9001/",
        timeout=10.0,
        verify_ssl=False,
        retry_attempts=2
    )


@pytest.fixture
def test_key_pair():
    """Generate test key pair."""
    return generate_key_pair()


@pytest.fixture
def temp_storage_dir():
    """Temporary directory for key storage tests."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield tmpdir


class TestServerConfig:
    """Test ServerConfig validation and functionality."""
    
    def test_valid_config(self):
        """Test creating valid server configuration."""
        config = ServerConfig(
            base_url="https://api.example.com",
            timeout=30.0,
            verify_ssl=True,
            retry_attempts=3
        )
        
        assert config.base_url == "https://api.example.com/"
        assert config.timeout == 30.0
        assert config.verify_ssl == True
        assert config.retry_attempts == 3
    
    def test_url_normalization(self):
        """Test URL normalization (adding trailing slash)."""
        config = ServerConfig(base_url="http://localhost:9001")
        assert config.base_url == "http://localhost:9001/"
    
    def test_invalid_url(self):
        """Test validation of invalid URLs."""
        with pytest.raises(ValidationError, match="Invalid server URL format"):
            ServerConfig(base_url="not-a-url")
    
    def test_invalid_timeout(self):
        """Test validation of invalid timeout."""
        with pytest.raises(ValidationError, match="Timeout must be positive"):
            ServerConfig(base_url="http://localhost:9001", timeout=-1)
    
    def test_invalid_retry_attempts(self):
        """Test validation of invalid retry attempts."""
        with pytest.raises(ValidationError, match="Retry attempts must be non-negative"):
            ServerConfig(base_url="http://localhost:9001", retry_attempts=-1)


class TestDataFoldHttpClient:
    """Test HTTP client functionality."""
    
    def test_client_creation(self, server_config, mock_requests):
        """Test creating HTTP client."""
        mock_requests_module, _, _ = mock_requests
        
        # Mock requests availability
        mock_requests_module.Session = MagicMock()
        
        client = DataFoldHttpClient(server_config)
        assert client.config == server_config
        assert hasattr(client, 'session')
    
    def test_client_creation_without_requests(self, server_config):
        """Test client creation fails without requests library."""
        with patch('datafold_sdk.http_client.REQUESTS_AVAILABLE', False):
            with pytest.raises(ServerCommunicationError, match="HTTP client requires 'requests' package"):
                DataFoldHttpClient(server_config)
    
    def test_register_public_key_success(self, server_config, test_key_pair, mock_requests):
        """Test successful public key registration."""
        mock_requests_module, mock_session, mock_response = mock_requests
        
        # Mock successful registration response
        mock_response.json.return_value = {
            'success': True,
            'data': {
                'registration_id': 'reg-123',
                'client_id': 'client-456',
                'public_key': test_key_pair.public_key.hex(),
                'key_name': 'test-key',
                'registered_at': '2023-06-08T12:00:00Z',
                'status': 'active'
            }
        }
        
        client = DataFoldHttpClient(server_config)
        registration = client.register_public_key(
            key_pair=test_key_pair,
            client_id='client-456',
            key_name='test-key'
        )
        
        assert isinstance(registration, PublicKeyRegistration)
        assert registration.registration_id == 'reg-123'
        assert registration.client_id == 'client-456'
        assert registration.status == 'active'
    
    def test_register_public_key_failure(self, server_config, test_key_pair, mock_requests):
        """Test public key registration failure."""
        mock_requests_module, mock_session, mock_response = mock_requests
        
        # Mock error response
        mock_response.json.return_value = {
            'success': False,
            'error': {
                'code': 'INVALID_PUBLIC_KEY',
                'message': 'Public key format is invalid'
            }
        }
        
        client = DataFoldHttpClient(server_config)
        
        with pytest.raises(ServerCommunicationError, match="Registration failed"):
            client.register_public_key(key_pair=test_key_pair)
    
    def test_get_key_status_success(self, server_config, mock_requests):
        """Test successful key status retrieval."""
        mock_requests_module, mock_session, mock_response = mock_requests
        
        # Mock successful status response
        mock_response.json.return_value = {
            'success': True,
            'data': {
                'registration_id': 'reg-123',
                'client_id': 'client-456',
                'public_key': 'abcd1234',
                'status': 'active',
                'registered_at': '2023-06-08T12:00:00Z'
            }
        }
        
        client = DataFoldHttpClient(server_config)
        status = client.get_key_status('client-456')
        
        assert isinstance(status, PublicKeyRegistration)
        assert status.client_id == 'client-456'
        assert status.status == 'active'
    
    def test_verify_signature_success(self, server_config, test_key_pair, mock_requests):
        """Test successful signature verification."""
        mock_requests_module, mock_session, mock_response = mock_requests
        
        # Create a signature
        message = "test message"
        signature = sign_message(test_key_pair.private_key, message)
        
        # Mock successful verification response
        mock_response.json.return_value = {
            'success': True,
            'data': {
                'verified': True,
                'client_id': 'client-456',
                'public_key': test_key_pair.public_key.hex(),
                'verified_at': '2023-06-08T12:00:00Z',
                'message_hash': 'sha256hash'
            }
        }
        
        client = DataFoldHttpClient(server_config)
        result = client.verify_signature(
            client_id='client-456',
            message=message,
            signature=signature
        )
        
        assert isinstance(result, SignatureVerificationResult)
        assert result.verified == True
        assert result.client_id == 'client-456'
    
    def test_verify_signature_failure(self, server_config, test_key_pair, mock_requests):
        """Test signature verification failure."""
        mock_requests_module, mock_session, mock_response = mock_requests
        
        # Mock verification failure response
        mock_response.json.return_value = {
            'success': False,
            'error': {
                'code': 'SIGNATURE_VERIFICATION_FAILED',
                'message': 'Digital signature verification failed'
            }
        }
        
        client = DataFoldHttpClient(server_config)
        result = client.verify_signature(
            client_id='client-456',
            message="test message",
            signature=b'x' * 64  # Invalid signature
        )
        
        assert isinstance(result, SignatureVerificationResult)
        assert result.verified == False
    
    def test_network_error_handling(self, server_config, mock_requests):
        """Test network error handling."""
        mock_requests_module, mock_session, mock_response = mock_requests
        
        # Mock network error
        import requests
        mock_session.request.side_effect = requests.exceptions.ConnectionError("Connection failed")
        
        client = DataFoldHttpClient(server_config)
        
        with pytest.raises(ServerCommunicationError, match="Connection error"):
            client.get_key_status('client-456')


class TestDataFoldClient:
    """Test high-level DataFold client integration."""
    
    def test_client_creation(self):
        """Test creating DataFold client."""
        with patch('datafold_sdk.integration.create_client') as mock_create:
            mock_http_client = MagicMock()
            mock_create.return_value = mock_http_client
            
            client = DataFoldClient("http://localhost:9001")
            
            assert client.server_url == "http://localhost:9001"
            assert client.http_client == mock_http_client
            mock_create.assert_called_once()
    
    def test_create_new_session_with_registration(self, temp_storage_dir):
        """Test creating new session with auto registration."""
        with patch('datafold_sdk.integration.create_client') as mock_create, \
             patch('datafold_sdk.integration.get_default_storage') as mock_storage:
            
            # Mock HTTP client
            mock_http_client = MagicMock()
            mock_create.return_value = mock_http_client
            
            # Mock successful registration
            mock_registration = PublicKeyRegistration(
                registration_id='reg-123',
                client_id='client-456',
                public_key='abcd1234',
                status='active'
            )
            mock_http_client.register_public_key.return_value = mock_registration
            
            # Mock storage
            mock_storage_instance = MagicMock()
            mock_storage.return_value = mock_storage_instance
            
            client = DataFoldClient("http://localhost:9001")
            session = client.create_new_session(
                client_id='client-456',
                key_name='test-key',
                auto_register=True,
                save_to_storage=True
            )
            
            assert isinstance(session, ClientSession)
            assert session.client_id == 'client-456'
            assert session.registration == mock_registration
            
            # Verify registration was called
            mock_http_client.register_public_key.assert_called_once()
            
            # Verify storage was called
            mock_storage_instance.store_key.assert_called_once()
    
    def test_create_new_session_registration_failure(self):
        """Test creating session when registration fails."""
        with patch('datafold_sdk.integration.create_client') as mock_create:
            
            # Mock HTTP client with registration failure
            mock_http_client = MagicMock()
            mock_create.return_value = mock_http_client
            mock_http_client.register_public_key.side_effect = ServerCommunicationError("Registration failed")
            
            client = DataFoldClient("http://localhost:9001")
            session = client.create_new_session(
                auto_register=True,
                save_to_storage=False
            )
            
            # Session should still be created despite registration failure
            assert isinstance(session, ClientSession)
            assert session.registration is None


class TestClientSession:
    """Test ClientSession functionality."""
    
    def test_session_creation(self, test_key_pair):
        """Test creating client session."""
        session = ClientSession(
            key_pair=test_key_pair,
            client_id='test-client'
        )
        
        assert session.key_pair == test_key_pair
        assert session.client_id == 'test-client'
        assert session.registration is None
    
    def test_sign_message(self, test_key_pair):
        """Test message signing in session."""
        session = ClientSession(
            key_pair=test_key_pair,
            client_id='test-client'
        )
        
        message = "test message"
        signature = session.sign_message(message)
        
        assert isinstance(signature, bytes)
        assert len(signature) == 64  # Ed25519 signature length
    
    def test_verify_with_server(self, test_key_pair):
        """Test server verification in session."""
        # Mock HTTP client
        mock_http_client = MagicMock()
        mock_result = SignatureVerificationResult(
            verified=True,
            client_id='test-client',
            public_key=test_key_pair.public_key.hex()
        )
        mock_http_client.verify_signature.return_value = mock_result
        
        session = ClientSession(
            key_pair=test_key_pair,
            client_id='test-client',
            http_client=mock_http_client
        )
        
        message = "test message"
        signature = session.sign_message(message)
        result = session.verify_with_server(message, signature)
        
        assert result.verified == True
        assert result.client_id == 'test-client'
        mock_http_client.verify_signature.assert_called_once()
    
    def test_verify_without_http_client(self, test_key_pair):
        """Test verification fails without HTTP client."""
        session = ClientSession(
            key_pair=test_key_pair,
            client_id='test-client'
        )
        
        with pytest.raises(ServerCommunicationError, match="No HTTP client configured"):
            session.verify_with_server("test", b'x' * 64)


class TestConvenienceFunctions:
    """Test convenience functions for quick setup."""
    
    def test_quick_setup(self):
        """Test quick setup function."""
        with patch('datafold_sdk.integration.DataFoldClient') as mock_client_class:
            mock_client = MagicMock()
            mock_session = MagicMock()
            mock_client.create_new_session.return_value = mock_session
            mock_client_class.return_value = mock_client
            
            session = quick_setup("http://localhost:9001", client_id="test-client")
            
            assert session == mock_session
            mock_client_class.assert_called_once_with("http://localhost:9001")
            mock_client.create_new_session.assert_called_once_with(client_id="test-client")


class TestEndToEndWorkflow:
    """Test complete end-to-end workflows."""
    
    def test_complete_workflow_mock(self):
        """Test complete workflow with mocked server responses."""
        with patch('datafold_sdk.integration.create_client') as mock_create:
            
            # Mock HTTP client
            mock_http_client = MagicMock()
            mock_create.return_value = mock_http_client
            
            # Mock registration
            test_key_pair = generate_key_pair()
            mock_registration = PublicKeyRegistration(
                registration_id='reg-123',
                client_id='client-456',
                public_key=test_key_pair.public_key.hex(),
                status='active'
            )
            mock_http_client.register_public_key.return_value = mock_registration
            
            # Mock verification
            mock_verification = SignatureVerificationResult(
                verified=True,
                client_id='client-456',
                public_key=test_key_pair.public_key.hex()
            )
            mock_http_client.verify_signature.return_value = mock_verification
            
            # Execute workflow
            client = DataFoldClient("http://localhost:9001")
            session = client.create_new_session(
                client_id='client-456',
                auto_register=True,
                save_to_storage=False
            )
            
            # Sign and verify message
            message = "Hello DataFold!"
            signature = session.sign_message(message)
            result = session.verify_with_server(message, signature)
            
            # Verify complete workflow
            assert session.registration.registration_id == 'reg-123'
            assert result.verified == True
            assert result.client_id == 'client-456'
            
            # Verify API calls
            mock_http_client.register_public_key.assert_called_once()
            mock_http_client.verify_signature.assert_called_once()


class TestErrorHandling:
    """Test error handling in various scenarios."""
    
    def test_invalid_key_pair_registration(self, server_config):
        """Test registration with invalid key pair."""
        with patch('datafold_sdk.http_client.REQUESTS_AVAILABLE', True), \
             patch('datafold_sdk.http_client.requests'):
            
            client = DataFoldHttpClient(server_config)
            
            with pytest.raises(ValidationError, match="key_pair must be an Ed25519KeyPair instance"):
                client.register_public_key("not-a-key-pair")
    
    def test_empty_client_id_verification(self, server_config):
        """Test verification with empty client ID."""
        with patch('datafold_sdk.http_client.REQUESTS_AVAILABLE', True), \
             patch('datafold_sdk.http_client.requests'):
            
            client = DataFoldHttpClient(server_config)
            
            with pytest.raises(ValidationError, match="client_id cannot be empty"):
                client.verify_signature("", "message", b'x' * 64)
    
    def test_invalid_signature_length(self, server_config):
        """Test verification with invalid signature length."""
        with patch('datafold_sdk.http_client.REQUESTS_AVAILABLE', True), \
             patch('datafold_sdk.http_client.requests'):
            
            client = DataFoldHttpClient(server_config)
            
            with pytest.raises(ValidationError, match="signature must be exactly 64 bytes"):
                client.verify_signature("client-123", "message", b'x' * 32)  # Wrong length


if __name__ == "__main__":
    pytest.main([__file__])