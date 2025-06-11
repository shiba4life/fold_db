"""
Signature verification module for DataFold Python SDK
Implements RFC 9421 HTTP Message Signatures verification

This module provides comprehensive signature verification capabilities including:
- Response signature verification
- Client-side verification tools
- Developer debugging utilities
- Integration and configuration tools
"""

# Export types
from .types import (
    VerificationStatus,
    VerificationPolicy,
    VerificationRule,
    VerificationRuleResult,
    VerificationConfig,
    KeySource,
    ExtractedSignatureData,
    VerificationContext,
    VerifiableResponse,
    VerificationResult,
    VerificationDiagnostics,
    VerificationError,
    SignatureInspector,
    SignatureFormatAnalysis,
    ComponentAnalysis,
    ParameterValidation,
    FormatIssue,
    ComponentIssue,
    ComponentSecurityAssessment,
    VerificationTestVector,
)

# Export policy constants
from .policies import (
    STRICT,
    STANDARD,
    LENIENT,
    LEGACY,
    get_verification_policy,
    get_available_verification_policies,
    VERIFICATION_POLICIES,
)

# Export core verifier
from .verifier import (
    RFC9421Verifier,
    create_verifier,
    verify_signature,
    verify_response,
    verify_request,
)

# Export inspector utilities
from .inspector import (
    RFC9421Inspector,
    create_inspector,
    validate_signature_format,
    quick_diagnostic,
    analyze_signature_security,
)

# Export middleware
from .middleware import (
    ResponseVerificationConfig,
    RequestVerificationConfig,
    create_response_verification_middleware,
    create_request_verification_middleware,
    create_batch_verifier,
    BatchVerifier,
)

# Export utilities
from .utils import (
    extract_signature_data,
    validate_signature_format as validate_format,
    reconstruct_canonical_message,
    verify_content_digest,
    check_timestamp_freshness,
    validate_nonce_uniqueness,
)

__all__ = [
    # Types
    'VerificationStatus',
    'VerificationPolicy',
    'VerificationRule',
    'VerificationRuleResult',
    'VerificationConfig',
    'KeySource',
    'ExtractedSignatureData',
    'VerificationContext',
    'VerifiableResponse',
    'VerificationResult',
    'VerificationDiagnostics',
    'VerificationError',
    'SignatureInspector',
    'SignatureFormatAnalysis',
    'ComponentAnalysis',
    'ParameterValidation',
    'FormatIssue',
    'ComponentIssue',
    'ComponentSecurityAssessment',
    'VerificationTestVector',
    
    # Policy Constants
    'STRICT',
    'STANDARD',
    'LENIENT',
    'LEGACY',
    'get_verification_policy',
    'get_available_verification_policies',
    'VERIFICATION_POLICIES',
    
    # Core verifier
    'RFC9421Verifier',
    'create_verifier',
    'verify_signature',
    'verify_response',
    'verify_request',
    
    # Inspector
    'RFC9421Inspector',
    'create_inspector',
    'validate_signature_format',
    'quick_diagnostic',
    'analyze_signature_security',
    
    # Middleware
    'ResponseVerificationConfig',
    'RequestVerificationConfig',
    'create_response_verification_middleware',
    'create_request_verification_middleware',
    'create_batch_verifier',
    'BatchVerifier',
    
    # Utils
    'extract_signature_data',
    'validate_format',
    'reconstruct_canonical_message',
    'verify_content_digest',
    'check_timestamp_freshness',
    'validate_nonce_uniqueness',
]