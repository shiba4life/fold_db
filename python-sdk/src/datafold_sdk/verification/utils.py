"""
Utility functions for signature verification

This module provides utility functions for RFC 9421 HTTP Message Signatures verification,
including signature data extraction, format validation, and content verification.
"""

import re
import time
import hashlib
import base64
from typing import Dict, List, Optional, Union, Any, Tuple

from .types import (
    ExtractedSignatureData,
    VerifiableResponse,
    VerificationError,
    VerificationErrorCodes,
    FormatIssue,
)
from ..signing.types import (
    SignableRequest,
    SignatureParams,
    SignatureAlgorithm,
    DigestAlgorithm,
    ContentDigest,
)
from ..signing.utils import (
    validate_nonce,
    validate_timestamp,
    validate_header_name,
    from_hex,
    PerformanceTimer,
)
from ..signing.canonical_message import build_canonical_message


def find_header_case_insensitive(headers: Dict[str, str], target_name: str) -> Optional[str]:
    """Find header with case-insensitive lookup"""
    target_lower = target_name.lower()
    for key, value in headers.items():
        if key.lower() == target_lower:
            return value
    return None


def extract_signature_data(headers: Dict[str, str]) -> ExtractedSignatureData:
    """
    Extract signature data from HTTP headers
    
    Args:
        headers: HTTP headers dictionary
        
    Returns:
        ExtractedSignatureData: Extracted signature information
        
    Raises:
        VerificationError: If signature data extraction fails
    """
    # Find signature-input and signature headers (case-insensitive)
    signature_input_header = find_header_case_insensitive(headers, 'signature-input')
    signature_header = find_header_case_insensitive(headers, 'signature')
    
    if not signature_input_header:
        raise VerificationError(
            'Signature-Input header not found',
            VerificationErrorCodes.MISSING_SIGNATURE_INPUT
        )
    
    if not signature_header:
        raise VerificationError(
            'Signature header not found',
            VerificationErrorCodes.MISSING_SIGNATURE
        )
    
    try:
        # Parse signature input
        parsed_input = parse_signature_input(signature_input_header)
        
        # Extract signature value
        signature_match = re.match(r'([^=]+)=:([^:]+):', signature_header)
        if not signature_match:
            raise VerificationError(
                'Invalid signature header format',
                VerificationErrorCodes.INVALID_SIGNATURE_FORMAT
            )
        
        signature_id, signature_value = signature_match.groups()
        
        if signature_id != parsed_input['signature_name']:
            raise VerificationError(
                f'Signature ID mismatch: {signature_id} != {parsed_input["signature_name"]}',
                VerificationErrorCodes.SIGNATURE_ID_MISMATCH
            )
        
        # Extract content digest if present
        content_digest = None
        content_digest_header = find_header_case_insensitive(headers, 'content-digest')
        if content_digest_header:
            digest_match = re.match(r'([^=]+)=:([^:]+):', content_digest_header)
            if digest_match:
                algorithm, value = digest_match.groups()
                content_digest = {
                    'algorithm': algorithm,
                    'value': value
                }
        
        return ExtractedSignatureData(
            signature_id=signature_id,
            signature=signature_value,
            covered_components=parsed_input['covered_components'],
            params=parsed_input['params'],
            content_digest=content_digest
        )
        
    except Exception as e:
        if isinstance(e, VerificationError):
            raise
        
        raise VerificationError(
            f'Failed to extract signature data: {e}',
            VerificationErrorCodes.INVALID_SIGNATURE_FORMAT,
            {'original_error': str(e)}
        )


def parse_signature_input(signature_input: str) -> Dict[str, Any]:
    """
    Parse Signature-Input header value according to RFC 9421
    
    Args:
        signature_input: Signature-Input header value
        
    Returns:
        dict: Parsed signature input data
        
    Raises:
        VerificationError: If parsing fails
    """
    try:
        # Pattern: sig1=("@method" "@target-uri" "content-type");created=123;keyid="key";alg="ed25519";nonce="nonce"
        match = re.match(r'([^=]+)=\(([^)]+)\);(.+)', signature_input)
        if not match:
            raise ValueError("Invalid signature input format")
        
        signature_name, components_str, params_str = match.groups()
        
        # Parse covered components
        covered_components = []
        for component in re.findall(r'"([^"]+)"', components_str):
            covered_components.append(component)
        
        # Parse parameters
        params_dict = {}
        for param_match in re.finditer(r'([^=;]+)=([^;]+)', params_str):
            key, value = param_match.groups()
            key = key.strip()
            value = value.strip()
            
            # Remove quotes if present
            if value.startswith('"') and value.endswith('"'):
                value = value[1:-1]
            
            params_dict[key] = value
        
        # Create signature parameters
        try:
            params = SignatureParams(
                created=int(params_dict['created']),
                keyid=params_dict['keyid'],
                alg=SignatureAlgorithm(params_dict['alg']),
                nonce=params_dict['nonce']
            )
        except (KeyError, ValueError) as e:
            raise ValueError(f"Invalid signature parameters: {e}")
        
        return {
            'signature_name': signature_name,
            'covered_components': covered_components,
            'params': params
        }
        
    except Exception as e:
        raise VerificationError(
            f'Failed to parse signature input: {e}',
            VerificationErrorCodes.INVALID_SIGNATURE_INPUT_FORMAT,
            {'signature_input': signature_input, 'original_error': str(e)}
        )


def validate_signature_format(headers: Dict[str, str]) -> Tuple[bool, List[FormatIssue]]:
    """
    Validate signature format according to RFC 9421
    
    Args:
        headers: HTTP headers to validate
        
    Returns:
        tuple: (is_valid, list of issues)
    """
    issues: List[FormatIssue] = []
    
    # Check for required headers
    signature_input = find_header_case_insensitive(headers, 'signature-input')
    signature = find_header_case_insensitive(headers, 'signature')
    
    if not signature_input:
        issues.append(FormatIssue(
            severity='error',
            code='MISSING_SIGNATURE_INPUT',
            message='Signature-Input header is required'
        ))
    
    if not signature:
        issues.append(FormatIssue(
            severity='error',
            code='MISSING_SIGNATURE',
            message='Signature header is required'
        ))
    
    # Validate signature-input format
    if signature_input:
        try:
            parse_signature_input(signature_input)
        except VerificationError as e:
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_SIGNATURE_INPUT_FORMAT',
                message=f'Invalid Signature-Input format: {e.message}'
            ))
    
    # Validate signature header format
    if signature:
        signature_pattern = re.compile(r'^[^=]+=:[0-9a-fA-F]+:$')
        if not signature_pattern.match(signature):
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_SIGNATURE_FORMAT',
                message='Signature header format is invalid (should be name=:hex:)'
            ))
        else:
            # Check signature length for Ed25519
            hex_match = re.search(r':([0-9a-fA-F]+):', signature)
            if hex_match:
                hex_signature = hex_match.group(1)
                if len(hex_signature) != 128:  # 64 bytes * 2 hex chars
                    issues.append(FormatIssue(
                        severity='warning',
                        code='UNEXPECTED_SIGNATURE_LENGTH',
                        message=f'Signature length is {len(hex_signature)} hex chars, expected 128 for Ed25519'
                    ))
    
    # Check content-digest format if present
    content_digest = find_header_case_insensitive(headers, 'content-digest')
    if content_digest:
        content_digest_pattern = re.compile(r'^[^=]+=:[A-Za-z0-9+/]+=*:$')
        if not content_digest_pattern.match(content_digest):
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_CONTENT_DIGEST_FORMAT',
                message='Content-Digest header format is invalid'
            ))
    
    # Overall validity
    error_count = len([issue for issue in issues if issue.severity == 'error'])
    is_valid = error_count == 0
    
    return is_valid, issues


def reconstruct_canonical_message(
    message: Union[SignableRequest, VerifiableResponse],
    signature_data: ExtractedSignatureData
) -> str:
    """
    Reconstruct canonical message for verification
    
    Args:
        message: Original message (request or response)
        signature_data: Extracted signature data
        
    Returns:
        str: Reconstructed canonical message
        
    Raises:
        VerificationError: If reconstruction fails
    """
    try:
        # Convert VerifiableResponse to SignableRequest if needed
        if isinstance(message, VerifiableResponse):
            signable_message = SignableRequest(
                method=message.method,
                url=message.url,
                headers=message.headers,
                body=message.body
            )
        else:
            signable_message = message
        
        # Create mock signing context for canonical message reconstruction
        from ..signing.types import SigningConfig, SignatureComponents, SigningContext, SigningOptions
        from ..signing.canonical_message import CanonicalMessageBuilder
        
        # Build signature components from covered components
        components = SignatureComponents(
            method='@method' in signature_data.covered_components,
            target_uri='@target-uri' in signature_data.covered_components,
            headers=[comp for comp in signature_data.covered_components if not comp.startswith('@') and comp != 'content-digest'],
            content_digest='content-digest' in signature_data.covered_components
        )
        
        # Create mock config (private key not needed for verification)
        config = SigningConfig(
            algorithm=signature_data.params.alg,
            key_id=signature_data.params.keyid,
            private_key=b'\x00' * 32,  # Dummy key for reconstruction
            components=components
        )
        
        # Create content digest if needed
        content_digest = None
        if signature_data.content_digest:
            from ..signing.types import ContentDigest
            content_digest = ContentDigest(
                algorithm=DigestAlgorithm(signature_data.content_digest['algorithm']),
                digest=signature_data.content_digest['value'],
                header_value=f"{signature_data.content_digest['algorithm']}=:{signature_data.content_digest['value']}:"
            )
        
        # Create signing context
        context = SigningContext(
            request=signable_message,
            config=config,
            options=SigningOptions(),
            params=signature_data.params,
            content_digest=content_digest
        )
        
        # Build canonical message
        builder = CanonicalMessageBuilder(context)
        return builder.build()
        
    except Exception as e:
        raise VerificationError(
            f'Failed to reconstruct canonical message: {e}',
            VerificationErrorCodes.CANONICAL_MESSAGE_RECONSTRUCTION_FAILED,
            {'original_error': str(e)}
        )


def verify_content_digest(
    content: Optional[Union[str, bytes]],
    expected_digest: Dict[str, str]
) -> bool:
    """
    Verify content digest matches expected value
    
    Args:
        content: Content to verify
        expected_digest: Expected digest info {'algorithm': str, 'value': str}
        
    Returns:
        bool: True if digest matches
    """
    if content is None:
        content = b""
    elif isinstance(content, str):
        content = content.encode('utf-8')
    
    try:
        algorithm = expected_digest['algorithm']
        expected_value = expected_digest['value']
        
        # Calculate digest
        if algorithm == 'sha-256':
            hasher = hashlib.sha256()
        elif algorithm == 'sha-512':
            hasher = hashlib.sha512()
        else:
            return False
        
        hasher.update(content)
        digest_bytes = hasher.digest()
        calculated_value = base64.b64encode(digest_bytes).decode('ascii')
        
        return calculated_value == expected_value
        
    except Exception:
        return False


def check_timestamp_freshness(timestamp: int, max_age: Optional[int] = None) -> bool:
    """
    Check if timestamp is within acceptable age
    
    Args:
        timestamp: Unix timestamp to check
        max_age: Maximum allowed age in seconds
        
    Returns:
        bool: True if timestamp is fresh
    """
    if not validate_timestamp(timestamp):
        return False
    
    if max_age is None:
        return True
    
    now = int(time.time())
    age = now - timestamp
    
    # Allow some clock skew (1 minute)
    if age < -60:
        return False
    
    return age <= max_age


def validate_nonce_uniqueness(nonce: str, used_nonces: set) -> bool:
    """
    Validate nonce uniqueness (for replay protection)
    
    Args:
        nonce: Nonce to validate
        used_nonces: Set of previously used nonces
        
    Returns:
        bool: True if nonce is unique
    """
    if not validate_nonce(nonce):
        return False
    
    if nonce in used_nonces:
        return False
    
    used_nonces.add(nonce)
    return True


def extract_signature_components_from_headers(headers: Dict[str, str]) -> List[str]:
    """
    Extract covered components from signature headers
    
    Args:
        headers: HTTP headers
        
    Returns:
        list: List of covered component names
    """
    try:
        signature_data = extract_signature_data(headers)
        return signature_data.covered_components
    except VerificationError:
        return []


def normalize_headers_for_verification(headers: Dict[str, str]) -> Dict[str, str]:
    """
    Normalize headers for verification (lowercase keys)
    
    Args:
        headers: Original headers
        
    Returns:
        dict: Normalized headers
    """
    return {key.lower(): value for key, value in headers.items()}


def get_content_size(message: Union[SignableRequest, VerifiableResponse]) -> int:
    """
    Get content size from message
    
    Args:
        message: Message to analyze
        
    Returns:
        int: Content size in bytes
    """
    if not hasattr(message, 'body') or message.body is None:
        return 0
    
    if isinstance(message.body, str):
        return len(message.body.encode('utf-8'))
    elif isinstance(message.body, bytes):
        return len(message.body)
    
    return 0


def get_content_type(message: Union[SignableRequest, VerifiableResponse]) -> Optional[str]:
    """
    Get content type from message headers
    
    Args:
        message: Message to analyze
        
    Returns:
        str or None: Content type if present
    """
    if not hasattr(message, 'headers'):
        return None
    
    return find_header_case_insensitive(message.headers, 'content-type')


def validate_signature_algorithm(algorithm: str, allowed_algorithms: List[str]) -> bool:
    """
    Validate signature algorithm against allowed list
    
    Args:
        algorithm: Algorithm to validate
        allowed_algorithms: List of allowed algorithms
        
    Returns:
        bool: True if algorithm is allowed
    """
    return algorithm in allowed_algorithms


def create_performance_timer() -> PerformanceTimer:
    """
    Create a performance timer for verification operations
    
    Returns:
        PerformanceTimer: Timer instance
    """
    return PerformanceTimer()


def format_verification_error(error: VerificationError) -> str:
    """
    Format verification error for display
    
    Args:
        error: Verification error
        
    Returns:
        str: Formatted error message
    """
    message = f"[{error.code}] {error.message}"
    if error.details:
        details_str = ", ".join(f"{k}={v}" for k, v in error.details.items())
        message += f" ({details_str})"
    return message


def is_pseudo_component(component: str) -> bool:
    """
    Check if component is a pseudo-component (starts with @)
    
    Args:
        component: Component name to check
        
    Returns:
        bool: True if pseudo-component
    """
    return component.startswith('@')


def validate_component_name(component: str) -> bool:
    """
    Validate component name format
    
    Args:
        component: Component name to validate
        
    Returns:
        bool: True if valid
    """
    if is_pseudo_component(component):
        # Pseudo-components have specific allowed values
        allowed_pseudo = ['@method', '@target-uri', '@authority', '@scheme', '@request-target']
        return component in allowed_pseudo
    else:
        # HTTP headers must follow RFC 7230
        return validate_header_name(component)