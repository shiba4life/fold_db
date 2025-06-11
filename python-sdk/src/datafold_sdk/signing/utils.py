"""
Utility functions for request signing

This module provides utility functions for RFC 9421 HTTP Message Signatures,
including nonce generation, timestamp handling, content digest calculation,
and URL parsing.
"""

import time
import uuid
import hashlib
import base64
import re
from typing import Dict, Tuple, Optional, Union
from urllib.parse import urlparse, parse_qs

from .types import (
    SigningError,
    SigningErrorCodes,
    DigestAlgorithm,
    ContentDigest,
    RequestBody,
)


def generate_nonce() -> str:
    """
    Generate a UUID v4 nonce for replay protection.
    
    Returns:
        str: UUID v4 string for use as nonce
        
    Raises:
        SigningError: If nonce generation fails
    """
    try:
        return str(uuid.uuid4())
    except Exception as e:
        raise SigningError(
            f"Failed to generate nonce: {e}",
            SigningErrorCodes.CRYPTO_ERROR,
            {"original_error": str(e)}
        )


def generate_timestamp() -> int:
    """
    Generate current Unix timestamp.
    
    Returns:
        int: Current Unix timestamp (seconds since epoch)
    """
    return int(time.time())


def format_rfc3339_timestamp(timestamp: Optional[int] = None) -> str:
    """
    Format timestamp as RFC 3339 string.
    
    Args:
        timestamp: Unix timestamp (uses current time if None)
        
    Returns:
        str: RFC 3339 formatted timestamp string
    """
    if timestamp is None:
        timestamp = generate_timestamp()
    
    # Convert to RFC 3339 format
    dt = time.gmtime(timestamp)
    return time.strftime('%Y-%m-%dT%H:%M:%SZ', dt)


def validate_nonce(nonce: str) -> bool:
    """
    Validate nonce format (should be UUID v4).
    
    Args:
        nonce: Nonce string to validate
        
    Returns:
        bool: True if nonce is valid UUID v4 format
    """
    if not isinstance(nonce, str):
        return False
    
    # UUID v4 pattern
    uuid_pattern = re.compile(
        r'^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$',
        re.IGNORECASE
    )
    
    return bool(uuid_pattern.match(nonce))


def validate_timestamp(timestamp: int) -> bool:
    """
    Validate timestamp (should be reasonable Unix timestamp).
    
    Args:
        timestamp: Unix timestamp to validate
        
    Returns:
        bool: True if timestamp is valid
    """
    if not isinstance(timestamp, int):
        return False
    
    # Check if it's a valid Unix timestamp (after 2000 and before 2100)
    year_2000 = 946684800  # 2000-01-01 00:00:00 UTC
    year_2100 = 4102444800  # 2100-01-01 00:00:00 UTC
    
    return year_2000 <= timestamp <= year_2100


def validate_signing_private_key(private_key: bytes) -> bool:
    """
    Validate Ed25519 private key format.
    
    Args:
        private_key: Private key bytes to validate
        
    Returns:
        bool: True if private key is valid
    """
    if not isinstance(private_key, bytes):
        return False
    
    if len(private_key) != 32:
        return False
    
    # Check for all-zero key (invalid)
    if private_key == b'\x00' * 32:
        return False
    
    return True


def parse_url(url: str) -> Dict[str, str]:
    """
    Parse URL to extract components needed for signing.
    
    Args:
        url: URL string to parse
        
    Returns:
        dict: Dictionary with parsed URL components:
            - origin: scheme + netloc
            - pathname: path component
            - search: query string (including ?)
            - target_uri: pathname + search
            
    Raises:
        SigningError: If URL format is invalid
    """
    try:
        parsed = urlparse(url)
        
        if not parsed.scheme or not parsed.netloc:
            raise SigningError(
                f"Invalid URL format: {url}",
                SigningErrorCodes.INVALID_URL,
                {"url": url}
            )
        
        # Only allow HTTP/HTTPS schemes for signing
        if parsed.scheme not in ('http', 'https'):
            raise SigningError(
                f"Unsupported URL scheme: {parsed.scheme}",
                SigningErrorCodes.INVALID_URL,
                {"url": url, "scheme": parsed.scheme}
            )
        
        origin = f"{parsed.scheme}://{parsed.netloc}"
        pathname = parsed.path or "/"
        search = f"?{parsed.query}" if parsed.query else ""
        target_uri = pathname + search
        
        return {
            "origin": origin,
            "pathname": pathname,
            "search": search,
            "target_uri": target_uri
        }
        
    except Exception as e:
        if isinstance(e, SigningError):
            raise
        
        raise SigningError(
            f"Failed to parse URL: {e}",
            SigningErrorCodes.INVALID_URL,
            {"url": url, "original_error": str(e)}
        )


def normalize_header_name(name: str) -> str:
    """
    Normalize header name to lowercase for consistent processing.
    
    Args:
        name: Header name to normalize
        
    Returns:
        str: Lowercase header name
    """
    return name.lower().strip()


def validate_header_name(name: str) -> bool:
    """
    Validate header name for inclusion in signature.
    
    Args:
        name: Header name to validate
        
    Returns:
        bool: True if header name is valid
    """
    if not isinstance(name, str):
        return False
    
    # RFC 7230 compliant header name validation
    header_name_pattern = re.compile(r'^[!#$%&\'*+\-.0-9A-Z^_`a-z|~]+$')
    return bool(header_name_pattern.match(name))


def calculate_content_digest(
    content: RequestBody,
    algorithm: DigestAlgorithm = DigestAlgorithm.SHA256
) -> ContentDigest:
    """
    Calculate content digest for request body.
    
    Args:
        content: Request body content (string, bytes, or None)
        algorithm: Digest algorithm to use
        
    Returns:
        ContentDigest: Calculated digest with header value
        
    Raises:
        SigningError: If digest calculation fails
    """
    if content is None:
        content = b""
    elif isinstance(content, str):
        content = content.encode('utf-8')
    elif not isinstance(content, bytes):
        raise SigningError(
            f"Content must be string, bytes, or None, got {type(content)}",
            SigningErrorCodes.DIGEST_CALCULATION_FAILED,
            {"content_type": str(type(content))}
        )
    
    try:
        # Select hash algorithm
        if algorithm == DigestAlgorithm.SHA256:
            hasher = hashlib.sha256()
        elif algorithm == DigestAlgorithm.SHA512:
            hasher = hashlib.sha512()
        else:
            raise SigningError(
                f"Unsupported digest algorithm: {algorithm}",
                SigningErrorCodes.DIGEST_CALCULATION_FAILED,
                {"algorithm": algorithm}
            )
        
        # Calculate digest
        hasher.update(content)
        digest_bytes = hasher.digest()
        
        # Base64 encode
        digest_b64 = base64.b64encode(digest_bytes).decode('ascii')
        
        # Format header value according to RFC 9421
        header_value = f"{algorithm.value}=:{digest_b64}:"
        
        return ContentDigest(
            algorithm=algorithm,
            digest=digest_b64,
            header_value=header_value
        )
        
    except Exception as e:
        if isinstance(e, SigningError):
            raise
        
        raise SigningError(
            f"Content digest calculation failed: {e}",
            SigningErrorCodes.DIGEST_CALCULATION_FAILED,
            {"algorithm": algorithm, "original_error": str(e)}
        )


def encode_signature_component(name: str, value: str) -> str:
    """
    Encode a signature component for canonical message.
    
    Args:
        name: Component name (e.g., "@method", "content-type")
        value: Component value
        
    Returns:
        str: Encoded component line for canonical message
    """
    # Escape any quotes in the value
    escaped_value = value.replace('"', '\\"')
    return f'"{name}": {escaped_value}'


def build_signature_params_string(
    covered_components: list,
    created: int,
    keyid: str,
    alg: str,
    nonce: str
) -> str:
    """
    Build @signature-params component string.
    
    Args:
        covered_components: List of covered component names
        created: Creation timestamp
        keyid: Key identifier
        alg: Algorithm name
        nonce: Nonce value
        
    Returns:
        str: @signature-params component string
    """
    # Format covered components list
    components_str = " ".join(f'"{comp}"' for comp in covered_components)
    
    return f"({components_str});created={created};keyid=\"{keyid}\";alg=\"{alg}\";nonce=\"{nonce}\""


def extract_covered_components(canonical_message: str) -> list:
    """
    Extract covered components from canonical message.
    
    Args:
        canonical_message: Canonical message string
        
    Returns:
        list: List of covered component names
    """
    lines = canonical_message.strip().split('\n')
    components = []
    
    for line in lines:
        if line.startswith('"') and '": ' in line:
            # Extract component name
            end_quote = line.find('"', 1)
            if end_quote > 0:
                component_name = line[1:end_quote]
                if component_name != "@signature-params":
                    components.append(component_name)
    
    return components


class PerformanceTimer:
    """Simple performance timer for monitoring signing operations."""
    
    def __init__(self):
        self.start_time = time.perf_counter()
    
    def elapsed_ms(self) -> float:
        """Get elapsed time in milliseconds."""
        return (time.perf_counter() - self.start_time) * 1000
    
    def elapsed_seconds(self) -> float:
        """Get elapsed time in seconds."""
        return time.perf_counter() - self.start_time
    
    def reset(self) -> None:
        """Reset the timer."""
        self.start_time = time.perf_counter()


def to_hex(data: bytes) -> str:
    """
    Convert bytes to lowercase hex string.
    
    Args:
        data: Bytes to convert
        
    Returns:
        str: Lowercase hex string
    """
    return data.hex().lower()


def from_hex(hex_string: str) -> bytes:
    """
    Convert hex string to bytes.
    
    Args:
        hex_string: Hex string to convert
        
    Returns:
        bytes: Converted bytes
        
    Raises:
        SigningError: If hex string is invalid
    """
    try:
        return bytes.fromhex(hex_string)
    except ValueError as e:
        raise SigningError(
            f"Invalid hex string: {e}",
            SigningErrorCodes.CRYPTO_ERROR,
            {"hex_string": hex_string}
        )


def safe_header_value(value: str) -> str:
    """
    Ensure header value is safe for HTTP headers.
    
    Args:
        value: Header value to sanitize
        
    Returns:
        str: Sanitized header value
    """
    # Remove any control characters and normalize whitespace
    sanitized = re.sub(r'[\x00-\x1f\x7f-\x9f]', '', value)
    sanitized = re.sub(r'\s+', ' ', sanitized)
    return sanitized.strip()