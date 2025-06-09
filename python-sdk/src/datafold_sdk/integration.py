"""
High-level integration module for DataFold SDK

This module provides simplified APIs that combine local key management
with server integration for end-to-end workflows.
"""

import logging
from typing import Dict, Optional, Any, Union, Tuple
from dataclasses import dataclass, field
import uuid

from .crypto.ed25519 import Ed25519KeyPair, generate_key_pair
from .crypto.storage import SecureKeyStorage, get_default_storage
from .http_client import (
    DataFoldHttpClient, ServerConfig, create_client,
    PublicKeyRegistration, SignatureVerificationResult
)
from .exceptions import (
    ServerCommunicationError, ValidationError, Ed25519KeyError,
    StorageError
)

logger = logging.getLogger(__name__)


@dataclass
class ClientSession:
    """
    Complete client session with key pair and server registration.
    
    Combines local key management with server integration to provide
    a complete client identity and authentication capability.
    """
    key_pair: Ed25519KeyPair
    client_id: str
    registration: Optional[PublicKeyRegistration] = None
    http_client: Optional[DataFoldHttpClient] = None
    storage: Optional[SecureKeyStorage] = None
    
    def sign_message(self, message: Union[str, bytes]) -> bytes:
        """
        Sign a message using the client's private key.
        
        Args:
            message: Message to sign (string or bytes)
            
        Returns:
            bytes: Ed25519 signature (64 bytes)
        """
        from .crypto.ed25519 import sign_message
        return sign_message(self.key_pair.private_key, message)
    
    def verify_with_server(
        self,
        message: Union[str, bytes],
        signature: bytes,
        message_encoding: str = 'utf8'
    ) -> SignatureVerificationResult:
        """
        Verify signature using the DataFold server.
        
        Args:
            message: Original message
            signature: Signature to verify
            message_encoding: Message encoding format
            
        Returns:
            SignatureVerificationResult: Verification result
            
        Raises:
            ServerCommunicationError: If no HTTP client configured
        """
        if not self.http_client:
            raise ServerCommunicationError("No HTTP client configured for server verification")
        
        return self.http_client.verify_signature(
            self.client_id, message, signature, message_encoding
        )
    
    def save_to_storage(self, key_name: Optional[str] = None) -> str:
        """
        Save the key pair to secure storage.
        
        Args:
            key_name: Optional name for the stored key
            
        Returns:
            str: Storage key identifier
            
        Raises:
            StorageError: If no storage configured
        """
        if not self.storage:
            raise StorageError("No storage configured for saving keys")
        
        storage_key = key_name or f"client_{self.client_id}"
        self.storage.store_key(storage_key, self.key_pair)
        logger.info(f"Key pair saved to storage with key: {storage_key}")
        return storage_key
    
    def close(self):
        """Close HTTP client and clean up resources."""
        if self.http_client:
            self.http_client.close()


class DataFoldClient:
    """
    High-level client for DataFold server integration.
    
    Provides simplified APIs for key generation, registration, and server
    communication with automatic retry and error handling.
    """
    
    def __init__(
        self,
        server_url: str,
        timeout: float = 30.0,
        verify_ssl: bool = True,
        retry_attempts: int = 3,
        storage: Optional[SecureKeyStorage] = None
    ):
        """
        Initialize DataFold client.
        
        Args:
            server_url: DataFold server base URL
            timeout: Request timeout in seconds
            verify_ssl: Whether to verify SSL certificates
            retry_attempts: Number of retry attempts
            storage: Optional secure storage instance
        """
        self.http_client = create_client(
            base_url=server_url,
            timeout=timeout,
            verify_ssl=verify_ssl,
            retry_attempts=retry_attempts
        )
        
        self.storage = storage or get_default_storage()
        self.server_url = server_url
        
        logger.info(f"DataFold client initialized for server: {server_url}")
    
    def create_new_session(
        self,
        client_id: Optional[str] = None,
        user_id: Optional[str] = None,
        key_name: Optional[str] = None,
        metadata: Optional[Dict[str, str]] = None,
        auto_register: bool = True,
        save_to_storage: bool = True
    ) -> ClientSession:
        """
        Create a new client session with fresh key pair.
        
        Args:
            client_id: Optional client identifier (generated if not provided)
            user_id: Optional user identifier
            key_name: Optional key name for storage and registration
            metadata: Optional metadata for server registration
            auto_register: Whether to automatically register with server
            save_to_storage: Whether to save key to local storage
            
        Returns:
            ClientSession: Complete client session
            
        Raises:
            Ed25519KeyError: On key generation failure
            ServerCommunicationError: On registration failure
            StorageError: On storage failure
        """
        # Generate new key pair
        key_pair = generate_key_pair()
        
        # Generate client ID if not provided
        if client_id is None:
            client_id = f"client_{uuid.uuid4().hex[:12]}"
        
        logger.info(f"Creating new session for client: {client_id}")
        
        # Create session
        session = ClientSession(
            key_pair=key_pair,
            client_id=client_id,
            http_client=self.http_client,
            storage=self.storage
        )
        
        # Register with server if requested
        if auto_register:
            try:
                registration = self.http_client.register_public_key(
                    key_pair=key_pair,
                    client_id=client_id,
                    user_id=user_id,
                    key_name=key_name,
                    metadata=metadata
                )
                session.registration = registration
                logger.info(f"Client registered with server: {registration.registration_id}")
            except ServerCommunicationError as e:
                logger.warning(f"Server registration failed: {e}")
                # Continue without registration if requested
                pass
        
        # Save to storage if requested
        if save_to_storage and self.storage:
            try:
                storage_key = key_name or f"client_{client_id}"
                session.save_to_storage(storage_key)
            except StorageError as e:
                logger.warning(f"Storage save failed: {e}")
                # Continue without storage
                pass
        
        return session
    
    def load_session_from_storage(
        self,
        storage_key: str,
        client_id: Optional[str] = None,
        auto_check_status: bool = True
    ) -> ClientSession:
        """
        Load existing client session from storage.
        
        Args:
            storage_key: Storage key for the saved key pair
            client_id: Optional client ID (derived from storage key if not provided)
            auto_check_status: Whether to check registration status with server
            
        Returns:
            ClientSession: Loaded client session
            
        Raises:
            StorageError: If key not found in storage
            ServerCommunicationError: On server communication failure
        """
        if not self.storage:
            raise StorageError("No storage configured for loading keys")
        
        # Load key pair from storage
        key_pair = self.storage.retrieve_key(storage_key)
        
        # Derive client ID if not provided
        if client_id is None:
            # Try to extract from storage key pattern
            if storage_key.startswith('client_'):
                client_id = storage_key
            else:
                client_id = f"client_{storage_key}"
        
        logger.info(f"Loading session for client: {client_id}")
        
        # Create session
        session = ClientSession(
            key_pair=key_pair,
            client_id=client_id,
            http_client=self.http_client,
            storage=self.storage
        )
        
        # Check registration status if requested
        if auto_check_status:
            try:
                registration = self.http_client.get_key_status(client_id)
                session.registration = registration
                logger.info(f"Found existing registration: {registration.registration_id}")
            except ServerCommunicationError as e:
                logger.warning(f"Could not verify server registration: {e}")
                # Continue without registration info
                pass
        
        return session
    
    def register_existing_key(
        self,
        key_pair: Ed25519KeyPair,
        client_id: str,
        user_id: Optional[str] = None,
        key_name: Optional[str] = None,
        metadata: Optional[Dict[str, str]] = None
    ) -> PublicKeyRegistration:
        """
        Register an existing key pair with the server.
        
        Args:
            key_pair: Existing Ed25519 key pair
            client_id: Client identifier
            user_id: Optional user identifier
            key_name: Optional key name
            metadata: Optional metadata
            
        Returns:
            PublicKeyRegistration: Registration confirmation
        """
        return self.http_client.register_public_key(
            key_pair=key_pair,
            client_id=client_id,
            user_id=user_id,
            key_name=key_name,
            metadata=metadata
        )
    
    def verify_signature(
        self,
        client_id: str,
        message: Union[str, bytes],
        signature: bytes,
        message_encoding: str = 'utf8'
    ) -> SignatureVerificationResult:
        """
        Verify signature using server verification.
        
        Args:
            client_id: Client identifier
            message: Original message
            signature: Signature to verify
            message_encoding: Message encoding format
            
        Returns:
            SignatureVerificationResult: Verification result
        """
        return self.http_client.verify_signature(
            client_id, message, signature, message_encoding
        )
    
    def list_stored_keys(self) -> list:
        """
        List all stored key identifiers.
        
        Returns:
            list: List of storage key identifiers
        """
        if not self.storage:
            return []
        
        # Get list of stored keys by scanning storage directory
        try:
            if hasattr(self.storage, 'storage_dir'):
                storage_dir = self.storage.storage_dir
                key_files = list(storage_dir.glob('*.key'))
                return [f.stem for f in key_files]
            else:
                return []
        except Exception:
            return []
    
    def close(self):
        """Close HTTP client and clean up resources."""
        self.http_client.close()


# Convenience functions

def quick_setup(
    server_url: str,
    client_id: Optional[str] = None,
    **kwargs
) -> ClientSession:
    """
    Quick setup for new client with server registration.
    
    Args:
        server_url: DataFold server URL
        client_id: Optional client ID
        **kwargs: Additional arguments for create_new_session
        
    Returns:
        ClientSession: Ready-to-use client session
    """
    client = DataFoldClient(server_url)
    return client.create_new_session(client_id=client_id, **kwargs)


def load_existing_client(
    server_url: str,
    storage_key: str,
    client_id: Optional[str] = None
) -> ClientSession:
    """
    Load existing client from storage.
    
    Args:
        server_url: DataFold server URL
        storage_key: Storage key for saved key pair
        client_id: Optional client ID
        
    Returns:
        ClientSession: Loaded client session
    """
    client = DataFoldClient(server_url)
    return client.load_session_from_storage(storage_key, client_id)


def register_and_verify_workflow(
    server_url: str,
    message: str = "test message",
    client_id: Optional[str] = None
) -> Tuple[ClientSession, SignatureVerificationResult]:
    """
    Complete workflow: generate key, register, sign message, and verify.
    
    This is useful for testing and demonstrating the full integration.
    
    Args:
        server_url: DataFold server URL
        message: Message to sign and verify
        client_id: Optional client ID
        
    Returns:
        tuple: (ClientSession, SignatureVerificationResult)
    """
    # Create client and session
    session = quick_setup(server_url, client_id=client_id)
    
    # Sign message
    signature = session.sign_message(message)
    
    # Verify with server
    result = session.verify_with_server(message, signature)
    
    logger.info(f"Complete workflow successful - verified: {result.verified}")
    
    return session, result