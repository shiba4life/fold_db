"""
Type definitions for signature verification functionality

This module provides type definitions and data classes for RFC 9421 HTTP Message Signatures
verification implementation with Ed25519 support.
"""

import time
from typing import Dict, List, Optional, Union, Callable, Any, Protocol, runtime_checkable
from dataclasses import dataclass, field
from enum import Enum
from abc import ABC, abstractmethod

from ..signing.types import SignableRequest, SignatureParams, DigestAlgorithm


class VerificationStatus(str, Enum):
    """Verification result status"""
    VALID = "valid"
    INVALID = "invalid"
    UNKNOWN = "unknown"
    ERROR = "error"


@dataclass
class VerificationRule:
    """Custom verification rule"""
    name: str
    description: str
    validate: Callable[['VerificationContext'], Union['VerificationRuleResult', Callable[[], 'VerificationRuleResult']]]
    
    def __post_init__(self):
        """Validate rule after initialization"""
        if not self.name:
            raise ValueError("Rule name cannot be empty")
        if not callable(self.validate):
            raise ValueError("Validate must be callable")


@dataclass
class VerificationRuleResult:
    """Verification rule result"""
    passed: bool
    message: Optional[str] = None
    details: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Initialize default details if None"""
        if self.details is None:
            self.details = {}


@dataclass
class VerificationPolicy:
    """Verification policy for different security levels"""
    name: str
    description: str
    verify_timestamp: bool = True
    max_timestamp_age: Optional[int] = None  # seconds
    verify_nonce: bool = True
    verify_content_digest: bool = True
    required_components: Optional[List[str]] = None
    allowed_algorithms: List[str] = field(default_factory=lambda: ['ed25519'])
    require_all_headers: bool = False
    custom_rules: Optional[List[VerificationRule]] = None
    
    def __post_init__(self):
        """Initialize defaults and validate policy"""
        if not self.name:
            raise ValueError("Policy name cannot be empty")
        if not self.description:
            raise ValueError("Policy description cannot be empty")
        if not self.allowed_algorithms:
            raise ValueError("At least one algorithm must be allowed")
        
        if self.required_components is None:
            self.required_components = []
        if self.custom_rules is None:
            self.custom_rules = []
            
        # Validate max_timestamp_age
        if self.max_timestamp_age is not None and self.max_timestamp_age <= 0:
            raise ValueError("Max timestamp age must be positive")


@runtime_checkable
class KeySource(Protocol):
    """Protocol for key source implementations"""
    
    name: str
    type: str  # 'url', 'function', 'cache'
    cache_ttl: Optional[int]
    
    async def retrieve_key(self, key_id: str) -> Optional[bytes]:
        """Retrieve public key by ID"""
        ...


@dataclass
class SimpleKeySource:
    """Simple key source implementation"""
    name: str
    type: str
    source: Union[str, Callable[[str], Optional[bytes]]]
    cache_ttl: Optional[int] = None
    
    async def retrieve_key(self, key_id: str) -> Optional[bytes]:
        """Retrieve public key by ID"""
        if callable(self.source):
            result = self.source(key_id)
            # Handle both sync and async callables
            if hasattr(result, '__await__'):
                return await result
            return result
        return None


@dataclass
class VerificationConfig:
    """Configuration for signature verification"""
    default_policy: Optional[str] = "standard"
    policies: Dict[str, VerificationPolicy] = field(default_factory=dict)
    public_keys: Dict[str, bytes] = field(default_factory=dict)  # keyId -> publicKey
    trusted_key_sources: Optional[List[KeySource]] = None
    performance_monitoring: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Initialize defaults and validate config"""
        if self.trusted_key_sources is None:
            self.trusted_key_sources = []
        
        if self.performance_monitoring is None:
            self.performance_monitoring = {
                'enabled': True,
                'max_verification_time': 50  # milliseconds
            }


@dataclass
class ExtractedSignatureData:
    """Signature data extracted from headers"""
    signature_id: str
    signature: str
    covered_components: List[str]
    params: SignatureParams
    content_digest: Optional[Dict[str, str]] = None  # {'algorithm': str, 'value': str}
    
    def __post_init__(self):
        """Validate extracted data"""
        if not self.signature_id:
            raise ValueError("Signature ID cannot be empty")
        if not self.signature:
            raise ValueError("Signature cannot be empty")
        if not self.covered_components:
            raise ValueError("Covered components cannot be empty")


@dataclass
class VerifiableResponse:
    """Response that can be verified"""
    status: int
    headers: Dict[str, str]
    body: Optional[Union[str, bytes]] = None
    url: str = ""
    method: str = "GET"
    
    def __post_init__(self):
        """Validate response data"""
        if not isinstance(self.status, int) or self.status < 100 or self.status >= 600:
            raise ValueError("Status must be a valid HTTP status code")
        if not isinstance(self.headers, dict):
            raise ValueError("Headers must be a dictionary")


@dataclass
class VerificationContext:
    """Verification context for signature validation"""
    message: Union[SignableRequest, VerifiableResponse]
    signature_data: ExtractedSignatureData
    policy: VerificationPolicy
    public_key: bytes
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Initialize defaults"""
        if self.metadata is None:
            self.metadata = {}


@dataclass
class VerificationDiagnostics:
    """Detailed diagnostic information"""
    signature_analysis: Dict[str, Any]
    content_analysis: Dict[str, Any]
    policy_compliance: Dict[str, Any]
    security_analysis: Dict[str, Any]
    
    @classmethod
    def create_empty(cls) -> 'VerificationDiagnostics':
        """Create empty diagnostics"""
        return cls(
            signature_analysis={},
            content_analysis={},
            policy_compliance={},
            security_analysis={}
        )


@dataclass
class VerificationResult:
    """Comprehensive verification result"""
    status: VerificationStatus
    signature_valid: bool
    checks: Dict[str, bool]
    diagnostics: VerificationDiagnostics
    performance: Dict[str, Any]
    error: Optional[Dict[str, Any]] = None
    
    @classmethod
    def create_error(cls, error_code: str, error_message: str, details: Optional[Dict[str, Any]] = None) -> 'VerificationResult':
        """Create error result"""
        return cls(
            status=VerificationStatus.ERROR,
            signature_valid=False,
            checks={
                'format_valid': False,
                'cryptographic_valid': False,
                'timestamp_valid': False,
                'nonce_valid': False,
                'content_digest_valid': False,
                'component_coverage_valid': False,
                'custom_rules_valid': False
            },
            diagnostics=VerificationDiagnostics.create_empty(),
            performance={'total_time': 0, 'step_timings': {}},
            error={
                'code': error_code,
                'message': error_message,
                'details': details or {}
            }
        )


class VerificationError(Exception):
    """Error class for verification operations"""
    
    def __init__(
        self,
        message: str,
        code: str,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(message)
        self.message = message
        self.code = code
        self.details = details or {}
    
    def __str__(self) -> str:
        if self.details:
            return f"{self.message} (code: {self.code}, details: {self.details})"
        return f"{self.message} (code: {self.code})"
    
    def __repr__(self) -> str:
        return f"VerificationError(message='{self.message}', code='{self.code}', details={self.details})"


# Format issue and component analysis types
@dataclass
class FormatIssue:
    """Format issue description"""
    severity: str  # 'error', 'warning', 'info'
    code: str
    message: str
    component: Optional[str] = None


@dataclass
class ComponentIssue:
    """Component issue description"""
    component: str
    type: str  # 'format', 'missing', 'unexpected', 'security'
    message: str
    suggestion: Optional[str] = None


@dataclass
class ComponentSecurityAssessment:
    """Component security assessment"""
    level: str  # 'low', 'medium', 'high'
    score: int  # 0-100
    strengths: List[str]
    weaknesses: List[str]
    
    def __post_init__(self):
        """Validate score"""
        if not 0 <= self.score <= 100:
            raise ValueError("Security score must be between 0 and 100")


@dataclass
class SignatureFormatAnalysis:
    """Signature format analysis result"""
    is_valid_rfc9421: bool
    issues: List[FormatIssue]
    signature_headers: List[str]
    signature_ids: List[str]


@dataclass
class ComponentAnalysis:
    """Component analysis result"""
    valid_components: List[str]
    invalid_components: List[ComponentIssue]
    missing_components: List[str]
    security_assessment: ComponentSecurityAssessment


@dataclass
class ParameterValidation:
    """Parameter validation result"""
    all_valid: bool
    parameters: Dict[str, Dict[str, Any]]  # parameter name -> {valid: bool, message?: str}
    insights: List[str]


@dataclass
class VerificationTestVector:
    """Test vector for verification testing"""
    name: str
    description: str
    category: str  # 'positive', 'negative', 'edge-case'
    input: Dict[str, Any]
    expected: Dict[str, Any]
    metadata: Optional[Dict[str, Any]] = None


# Abstract base classes and protocols
class SignatureInspector(ABC):
    """Abstract signature inspector interface for debugging"""
    
    @abstractmethod
    def inspect_format(self, headers: Dict[str, str]) -> SignatureFormatAnalysis:
        """Inspect signature format and structure"""
        pass
    
    @abstractmethod
    def analyze_components(self, signature_data: ExtractedSignatureData) -> ComponentAnalysis:
        """Analyze signature components"""
        pass
    
    @abstractmethod
    def validate_parameters(self, params: SignatureParams) -> ParameterValidation:
        """Validate signature parameters"""
        pass
    
    @abstractmethod
    def generate_diagnostic_report(self, result: VerificationResult) -> str:
        """Generate diagnostic report"""
        pass


# Type aliases for convenience
VerificationRuleValidator = Callable[[VerificationContext], Union[VerificationRuleResult, Callable[[], VerificationRuleResult]]]
HeaderDict = Dict[str, str]
KeyRetriever = Callable[[str], Optional[bytes]]
AsyncKeyRetriever = Callable[[str], Callable[[], Optional[bytes]]]


# Common verification error codes
class VerificationErrorCodes:
    """Standard error codes for verification operations"""
    
    # Configuration errors
    INVALID_CONFIG = "INVALID_CONFIG"
    INVALID_POLICY = "INVALID_POLICY"
    UNKNOWN_POLICY = "UNKNOWN_POLICY"
    
    # Signature format errors
    MISSING_SIGNATURE_INPUT = "MISSING_SIGNATURE_INPUT"
    MISSING_SIGNATURE = "MISSING_SIGNATURE"
    INVALID_SIGNATURE_FORMAT = "INVALID_SIGNATURE_FORMAT"
    INVALID_SIGNATURE_INPUT_FORMAT = "INVALID_SIGNATURE_INPUT_FORMAT"
    SIGNATURE_ID_MISMATCH = "SIGNATURE_ID_MISMATCH"
    
    # Key management errors
    PUBLIC_KEY_NOT_FOUND = "PUBLIC_KEY_NOT_FOUND"
    KEY_RETRIEVAL_FAILED = "KEY_RETRIEVAL_FAILED"
    INVALID_PUBLIC_KEY = "INVALID_PUBLIC_KEY"
    
    # Cryptographic errors
    CRYPTOGRAPHIC_VERIFICATION_FAILED = "CRYPTOGRAPHIC_VERIFICATION_FAILED"
    SIGNATURE_VERIFICATION_FAILED = "SIGNATURE_VERIFICATION_FAILED"
    
    # Validation errors
    TIMESTAMP_VALIDATION_FAILED = "TIMESTAMP_VALIDATION_FAILED"
    NONCE_VALIDATION_FAILED = "NONCE_VALIDATION_FAILED"
    CONTENT_DIGEST_VALIDATION_FAILED = "CONTENT_DIGEST_VALIDATION_FAILED"
    COMPONENT_COVERAGE_FAILED = "COMPONENT_COVERAGE_FAILED"
    CUSTOM_RULE_FAILED = "CUSTOM_RULE_FAILED"
    
    # General errors
    VERIFICATION_FAILED = "VERIFICATION_FAILED"
    CANONICAL_MESSAGE_RECONSTRUCTION_FAILED = "CANONICAL_MESSAGE_RECONSTRUCTION_FAILED"
    MIDDLEWARE_ERROR = "MIDDLEWARE_ERROR"
    PERFORMANCE_TIMEOUT = "PERFORMANCE_TIMEOUT"