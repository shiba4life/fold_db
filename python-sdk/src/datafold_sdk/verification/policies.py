"""
Predefined verification policies for different security levels

This module provides built-in verification policies and custom verification rules
for RFC 9421 HTTP Message Signatures verification.
"""

import time
from typing import Dict, List, Optional, Set, Union
from dataclasses import dataclass, field

from .types import (
    VerificationPolicy,
    VerificationRule,
    VerificationRuleResult,
    VerificationContext,
    VerificationError,
    VerificationErrorCodes,
)
from ..signing.utils import validate_nonce, validate_timestamp


def create_replay_protection_rule(nonce_cache: Optional[Set[str]] = None) -> VerificationRule:
    """Create replay protection rule with nonce validation"""
    if nonce_cache is None:
        nonce_cache = set()
    
    def validate_replay_protection(context: VerificationContext) -> VerificationRuleResult:
        """Ensure nonce is fresh and not reused"""
        nonce = context.signature_data.params.nonce
        
        if not validate_nonce(nonce):
            return VerificationRuleResult(
                passed=False,
                message='Invalid nonce format',
                details={'nonce': nonce}
            )
        
        # Check for replay (note: in production, use distributed nonce cache)
        if nonce in nonce_cache:
            return VerificationRuleResult(
                passed=False,
                message=f'Nonce already used: {nonce}',
                details={'nonce': nonce}
            )
        
        nonce_cache.add(nonce)
        return VerificationRuleResult(
            passed=True,
            message='Nonce validation passed'
        )
    
    return VerificationRule(
        name='replay-protection',
        description='Ensure nonce is fresh and not reused',
        validate=validate_replay_protection
    )


def create_algorithm_strength_rule() -> VerificationRule:
    """Create algorithm strength validation rule"""
    
    def validate_algorithm_strength(context: VerificationContext) -> VerificationRuleResult:
        """Ensure strong cryptographic algorithm"""
        algorithm = context.signature_data.params.alg.value
        strong_algorithms = ['ed25519']
        
        if algorithm not in strong_algorithms:
            return VerificationRuleResult(
                passed=False,
                message=f'Weak algorithm detected: {algorithm}',
                details={'algorithm': algorithm, 'strong_algorithms': strong_algorithms}
            )
        
        return VerificationRuleResult(
            passed=True,
            message='Algorithm strength validated'
        )
    
    return VerificationRule(
        name='algorithm-strength',
        description='Ensure strong cryptographic algorithm',
        validate=validate_algorithm_strength
    )


def create_basic_replay_protection_rule() -> VerificationRule:
    """Create basic nonce format validation rule"""
    
    def validate_basic_replay_protection(context: VerificationContext) -> VerificationRuleResult:
        """Basic nonce format validation"""
        nonce = context.signature_data.params.nonce
        
        if not validate_nonce(nonce):
            return VerificationRuleResult(
                passed=False,
                message='Invalid nonce format',
                details={'nonce': nonce}
            )
        
        return VerificationRuleResult(
            passed=True,
            message='Nonce format validated'
        )
    
    return VerificationRule(
        name='basic-replay-protection',
        description='Basic nonce format validation',
        validate=validate_basic_replay_protection
    )


# Predefined verification policies
STRICT_VERIFICATION_POLICY = VerificationPolicy(
    name='strict',
    description='Maximum security verification with comprehensive checks',
    verify_timestamp=True,
    max_timestamp_age=300,  # 5 minutes
    verify_nonce=True,
    verify_content_digest=True,
    required_components=['@method', '@target-uri', 'content-type', 'content-digest'],
    allowed_algorithms=['ed25519'],
    require_all_headers=True,
    custom_rules=[
        create_replay_protection_rule(),
        create_algorithm_strength_rule(),
    ]
)

STANDARD_VERIFICATION_POLICY = VerificationPolicy(
    name='standard',
    description='Balanced verification suitable for most applications',
    verify_timestamp=True,
    max_timestamp_age=900,  # 15 minutes
    verify_nonce=True,
    verify_content_digest=True,
    required_components=['@method', '@target-uri'],
    allowed_algorithms=['ed25519'],
    require_all_headers=False,
    custom_rules=[
        create_basic_replay_protection_rule(),
    ]
)

LENIENT_VERIFICATION_POLICY = VerificationPolicy(
    name='lenient',
    description='Relaxed verification for development and testing',
    verify_timestamp=True,
    max_timestamp_age=3600,  # 1 hour
    verify_nonce=False,
    verify_content_digest=False,
    required_components=['@method'],
    allowed_algorithms=['ed25519'],
    require_all_headers=False,
    custom_rules=[]
)

LEGACY_VERIFICATION_POLICY = VerificationPolicy(
    name='legacy',
    description='Legacy verification for older signature formats',
    verify_timestamp=False,
    verify_nonce=False,
    verify_content_digest=False,
    required_components=[],
    allowed_algorithms=['ed25519'],
    require_all_headers=False,
    custom_rules=[]
)

# Verification policies registry
VERIFICATION_POLICIES: Dict[str, VerificationPolicy] = {
    'strict': STRICT_VERIFICATION_POLICY,
    'standard': STANDARD_VERIFICATION_POLICY,
    'lenient': LENIENT_VERIFICATION_POLICY,
    'legacy': LEGACY_VERIFICATION_POLICY,
}


class VerificationRules:
    """Custom verification rules library"""
    
    @staticmethod
    def timestamp_freshness(max_age: int) -> VerificationRule:
        """Timestamp freshness rule"""
        def validate_freshness(context: VerificationContext) -> VerificationRuleResult:
            now = int(time.time())
            created = context.signature_data.params.created
            age = now - created
            
            if age > max_age:
                return VerificationRuleResult(
                    passed=False,
                    message=f'Timestamp too old: {age}s > {max_age}s',
                    details={'age': age, 'max_age': max_age, 'created': created, 'now': now}
                )
            
            if age < -60:  # Allow 1 minute clock skew
                return VerificationRuleResult(
                    passed=False,
                    message=f'Timestamp from future: {age}s',
                    details={'age': age, 'created': created, 'now': now}
                )
            
            return VerificationRuleResult(
                passed=True,
                message=f'Timestamp is fresh: {age}s old'
            )
        
        return VerificationRule(
            name='timestamp-freshness',
            description=f'Ensure timestamp is within {max_age} seconds',
            validate=validate_freshness
        )
    
    @staticmethod
    def required_headers(headers: List[str]) -> VerificationRule:
        """Required headers rule"""
        def validate_required(context: VerificationContext) -> VerificationRuleResult:
            covered_components = context.signature_data.covered_components
            missing = [h for h in headers if h.lower() not in [c.lower() for c in covered_components]]
            
            if missing:
                return VerificationRuleResult(
                    passed=False,
                    message=f'Missing required headers: {", ".join(missing)}',
                    details={'missing': missing, 'required': headers, 'covered': covered_components}
                )
            
            return VerificationRuleResult(
                passed=True,
                message='All required headers present'
            )
        
        return VerificationRule(
            name='required-headers',
            description=f'Ensure required headers are present: {", ".join(headers)}',
            validate=validate_required
        )
    
    @staticmethod
    def key_id_validation(valid_key_ids: List[str]) -> VerificationRule:
        """Key ID validation rule"""
        def validate_key_id(context: VerificationContext) -> VerificationRuleResult:
            key_id = context.signature_data.params.keyid
            
            if key_id not in valid_key_ids:
                return VerificationRuleResult(
                    passed=False,
                    message=f'Invalid key ID: {key_id}',
                    details={'key_id': key_id, 'valid_key_ids': valid_key_ids}
                )
            
            return VerificationRuleResult(
                passed=True,
                message='Key ID validated'
            )
        
        return VerificationRule(
            name='key-id-validation',
            description='Validate key ID against allowed list',
            validate=validate_key_id
        )
    
    @staticmethod
    def content_type_consistency() -> VerificationRule:
        """Content type consistency rule"""
        def validate_consistency(context: VerificationContext) -> VerificationRuleResult:
            message = context.message
            has_body = hasattr(message, 'body') and getattr(message, 'body') is not None
            covered_components = context.signature_data.covered_components
            has_content_type = 'content-type' in [c.lower() for c in covered_components]
            
            if has_body and not has_content_type:
                return VerificationRuleResult(
                    passed=False,
                    message='Content-type should be covered when body is present',
                    details={'has_body': has_body, 'has_content_type': has_content_type}
                )
            
            return VerificationRuleResult(
                passed=True,
                message='Content-type coverage is appropriate'
            )
        
        return VerificationRule(
            name='content-type-consistency',
            description='Ensure content-type is covered when body is present',
            validate=validate_consistency
        )
    
    @staticmethod
    def nonce_uniqueness(nonce_cache: Union[Set[str], Dict[str, int]]) -> VerificationRule:
        """Nonce uniqueness rule (requires external storage)"""
        def validate_uniqueness(context: VerificationContext) -> VerificationRuleResult:
            nonce = context.signature_data.params.nonce
            
            if isinstance(nonce_cache, set):
                if nonce in nonce_cache:
                    return VerificationRuleResult(
                        passed=False,
                        message=f'Nonce already used: {nonce}',
                        details={'nonce': nonce}
                    )
                nonce_cache.add(nonce)
            elif isinstance(nonce_cache, dict):
                last_used = nonce_cache.get(nonce)
                if last_used:
                    return VerificationRuleResult(
                        passed=False,
                        message=f'Nonce already used at: {time.ctime(last_used)}',
                        details={'nonce': nonce, 'last_used': last_used}
                    )
                nonce_cache[nonce] = int(time.time())
            
            return VerificationRuleResult(
                passed=True,
                message='Nonce is unique'
            )
        
        return VerificationRule(
            name='nonce-uniqueness',
            description='Ensure nonce has not been used before',
            validate=validate_uniqueness
        )


# Expose rules as module-level constant
VERIFICATION_RULES = VerificationRules


def create_verification_policy(
    name: str,
    description: str,
    **options
) -> VerificationPolicy:
    """Create a custom verification policy"""
    defaults = {
        'verify_timestamp': True,
        'verify_nonce': True,
        'verify_content_digest': True,
        'allowed_algorithms': ['ed25519'],
        'require_all_headers': False,
        'custom_rules': [],
    }
    
    # Merge defaults with provided options
    policy_kwargs = {**defaults, **options}
    
    return VerificationPolicy(
        name=name,
        description=description,
        **policy_kwargs
    )


def merge_verification_policies(
    base: VerificationPolicy,
    override: VerificationPolicy
) -> VerificationPolicy:
    """Merge verification policies"""
    custom_rules = []
    if base.custom_rules:
        custom_rules.extend(base.custom_rules)
    if override.custom_rules:
        custom_rules.extend(override.custom_rules)
    
    # Create new policy with merged attributes
    return VerificationPolicy(
        name=override.name or base.name,
        description=override.description or base.description,
        verify_timestamp=override.verify_timestamp if override.verify_timestamp is not None else base.verify_timestamp,
        max_timestamp_age=override.max_timestamp_age or base.max_timestamp_age,
        verify_nonce=override.verify_nonce if override.verify_nonce is not None else base.verify_nonce,
        verify_content_digest=override.verify_content_digest if override.verify_content_digest is not None else base.verify_content_digest,
        required_components=override.required_components or base.required_components,
        allowed_algorithms=override.allowed_algorithms or base.allowed_algorithms,
        require_all_headers=override.require_all_headers if override.require_all_headers is not None else base.require_all_headers,
        custom_rules=custom_rules
    )


def get_verification_policy(name: str) -> Optional[VerificationPolicy]:
    """Get verification policy by name"""
    return VERIFICATION_POLICIES.get(name)


def get_available_verification_policies() -> List[str]:
    """Get all available verification policies"""
    return list(VERIFICATION_POLICIES.keys())


def validate_verification_policy(policy: VerificationPolicy) -> None:
    """Validate verification policy configuration"""
    if not policy.name or not isinstance(policy.name, str):
        raise VerificationError(
            'Policy name must be a non-empty string',
            VerificationErrorCodes.INVALID_POLICY
        )
    
    if not policy.description or not isinstance(policy.description, str):
        raise VerificationError(
            'Policy description must be a non-empty string',
            VerificationErrorCodes.INVALID_POLICY
        )
    
    if not policy.allowed_algorithms or not isinstance(policy.allowed_algorithms, list):
        raise VerificationError(
            'Policy must specify at least one allowed algorithm',
            VerificationErrorCodes.INVALID_POLICY
        )
    
    if policy.max_timestamp_age is not None and policy.max_timestamp_age <= 0:
        raise VerificationError(
            'Maximum timestamp age must be positive',
            VerificationErrorCodes.INVALID_POLICY
        )
    
    if policy.custom_rules:
        for rule in policy.custom_rules:
            if not rule.name or not isinstance(rule.name, str):
                raise VerificationError(
                    'Custom rule name must be a non-empty string',
                    VerificationErrorCodes.INVALID_POLICY
                )
            if not callable(rule.validate):
                raise VerificationError(
                    'Custom rule must have a validate function',
                    VerificationErrorCodes.INVALID_POLICY
                )


def register_verification_policy(policy: VerificationPolicy) -> None:
    """Register a new verification policy"""
    validate_verification_policy(policy)
    VERIFICATION_POLICIES[policy.name] = policy


def unregister_verification_policy(name: str) -> bool:
    """Unregister a verification policy"""
    if name in VERIFICATION_POLICIES:
        del VERIFICATION_POLICIES[name]
        return True
    return False