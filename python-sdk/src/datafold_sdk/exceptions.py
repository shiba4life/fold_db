"""
Exception classes for DataFold Python SDK
"""

from typing import Optional, Dict, Any


class DataFoldSDKError(Exception):
    """Base exception for all DataFold SDK errors"""
    
    def __init__(self, message: str, error_code: str = "UNKNOWN_ERROR", details: Optional[Dict[str, Any]] = None):
        super().__init__(message)
        self.error_code = error_code
        self.details = details or {}


class Ed25519KeyError(DataFoldSDKError):
    """Exception raised for Ed25519 key generation or validation errors"""
    pass


class ValidationError(DataFoldSDKError):
    """Exception raised for validation failures"""
    pass


class UnsupportedPlatformError(DataFoldSDKError):
    """Exception raised when platform features are not supported"""
    pass


class StorageError(DataFoldSDKError):
    """Exception raised for key storage related errors"""
    pass


class KeyDerivationError(DataFoldSDKError):
    """Exception raised for key derivation errors"""
    pass


class KeyRotationError(DataFoldSDKError):
    """Exception raised for key rotation errors"""
    pass


class KeyExportError(DataFoldSDKError):
    """Exception raised for key export errors"""
    pass


class KeyImportError(DataFoldSDKError):
    """Exception raised for key import errors"""
    pass


class BackupError(DataFoldSDKError):
    """Exception raised for key backup and restore errors"""
    pass


class ServerCommunicationError(DataFoldSDKError):
    """Exception raised for server communication errors"""
    
    def __init__(self, message: str, error_code: str = "SERVER_ERROR", 
                 http_status: int = 0, details: Optional[Dict[str, Any]] = None):
        super().__init__(message, error_code, details)
        self.http_status = http_status