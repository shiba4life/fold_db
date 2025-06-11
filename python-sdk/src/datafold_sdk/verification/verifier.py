"""
Core signature verification engine for RFC 9421 HTTP Message Signatures

This module provides the main verifier implementation for RFC 9421 HTTP Message Signatures
using Ed25519 digital signatures, integrated with the existing DataFold SDK infrastructure.
"""

import asyncio
import time
from typing import Dict, List, Optional, Union, Any, Set
from dataclasses import dataclass

from .types import (
    VerificationResult,
    VerificationConfig,
    VerificationPolicy,
    VerificationContext,
    ExtractedSignatureData,
    VerificationError,
    VerifiableResponse,
    VerificationStatus,
    VerificationDiagnostics,
    KeySource,
    VerificationErrorCodes,
    VerificationRuleResult,
)
from .policies import VERIFICATION_POLICIES, get_verification_policy
from .utils import (
    extract_signature_data,
    find_header_case_insensitive,
    reconstruct_canonical_message,
    verify_content_digest,
    check_timestamp_freshness,
    validate_signature_algorithm,
    get_content_size,
    get_content_type,
    create_performance_timer,
)
from ..signing.types import SignableRequest, SignatureParams
from ..signing.utils import validate_nonce, validate_timestamp, from_hex
from ..crypto.ed25519 import verify_signature


class RFC9421Verifier:
    """
    RFC 9421 HTTP Message Signatures verifier with Ed25519
    
    This class provides signature verification functionality according to RFC 9421
    HTTP Message Signatures specification using Ed25519 digital signatures.
    """
    
    def __init__(self, config: VerificationConfig):
        """
        Initialize the verifier with configuration.
        
        Args:
            config: Verification configuration
            
        Raises:
            VerificationError: If configuration is invalid
        """
        self.config = self._copy_config(config)
        self.key_cache: Dict[str, Dict[str, Any]] = {}
        self.nonce_cache: Set[str] = set()
        self._validate_config()
    
    async def verify(
        self,
        message: Union[SignableRequest, VerifiableResponse],
        headers: Dict[str, str],
        policy: Optional[str] = None,
        public_key: Optional[bytes] = None,
        key_id: Optional[str] = None,
        skip_key_retrieval: bool = False
    ) -> VerificationResult:
        """
        Verify a signed request or response
        
        Args:
            message: Request or response to verify
            headers: HTTP headers containing signature
            policy: Verification policy name (optional)
            public_key: Public key for verification (optional)
            key_id: Key ID override (optional)
            skip_key_retrieval: Skip external key retrieval
            
        Returns:
            VerificationResult: Comprehensive verification result
        """
        timer = create_performance_timer()
        step_timings: Dict[str, float] = {}
        
        try:
            # Step 1: Extract signature data from headers
            timer.reset()
            signature_data = extract_signature_data(headers)
            step_timings['extraction'] = timer.elapsed_ms()
            
            # Step 2: Get verification policy
            timer.reset()
            verification_policy = self._get_policy(policy)
            step_timings['policy_retrieval'] = timer.elapsed_ms()
            
            # Step 3: Get public key for verification
            timer.reset()
            effective_key_id = key_id or signature_data.params.keyid
            verification_public_key = await self._get_public_key(
                effective_key_id,
                public_key,
                skip_key_retrieval
            )
            step_timings['key_retrieval'] = timer.elapsed_ms()
            
            # Step 4: Create verification context
            context = VerificationContext(
                message=message,
                signature_data=signature_data,
                policy=verification_policy,
                public_key=verification_public_key,
                metadata={
                    'headers': headers,
                    'policy_name': policy,
                    'key_id': effective_key_id
                }
            )
            
            # Step 5: Perform comprehensive verification
            timer.reset()
            result = await self._perform_verification(context)
            step_timings['verification'] = timer.elapsed_ms()
            
            # Add performance metrics
            total_time = sum(step_timings.values())
            result.performance = {
                'total_time': total_time,
                'step_timings': step_timings
            }
            
            # Performance warning if too slow
            max_time = self.config.performance_monitoring.get('max_verification_time', 50)
            if total_time > max_time:
                import logging
                logger = logging.getLogger(__name__)
                logger.warning(f"Verification took {total_time:.2f}ms (target: <{max_time}ms)")
            
            return result
            
        except Exception as error:
            total_time = timer.elapsed_ms()
            
            if isinstance(error, VerificationError):
                return self._create_error_result(error, total_time, step_timings)
            
            verification_error = VerificationError(
                f"Verification failed: {error}",
                VerificationErrorCodes.VERIFICATION_FAILED,
                {'original_error': str(error)}
            )
            
            return self._create_error_result(verification_error, total_time, step_timings)
    
    async def verify_batch(
        self,
        verifications: List[Dict[str, Any]]
    ) -> List[VerificationResult]:
        """
        Verify multiple signatures (batch verification)
        
        Args:
            verifications: List of verification requests
            
        Returns:
            List[VerificationResult]: Verification results
        """
        results = []
        
        # Process in parallel for better performance
        tasks = []
        for verification in verifications:
            task = self.verify(
                verification['message'],
                verification['headers'],
                verification.get('policy'),
                verification.get('public_key'),
                verification.get('key_id'),
                verification.get('skip_key_retrieval', False)
            )
            tasks.append(task)
        
        batch_results = await asyncio.gather(*tasks, return_exceptions=True)
        
        for result in batch_results:
            if isinstance(result, VerificationResult):
                results.append(result)
            elif isinstance(result, Exception):
                error_result = VerificationResult.create_error(
                    'BATCH_VERIFICATION_ERROR',
                    f'Batch verification failed: {result}'
                )
                results.append(error_result)
            else:
                error_result = VerificationResult.create_error(
                    'BATCH_VERIFICATION_ERROR',
                    'Unknown batch verification error'
                )
                results.append(error_result)
        
        return results
    
    def update_config(self, new_config: VerificationConfig) -> None:
        """
        Update verification configuration
        
        Args:
            new_config: New configuration (will be merged with existing)
        """
        # Merge configurations
        merged_config = VerificationConfig(
            default_policy=new_config.default_policy or self.config.default_policy,
            policies={**self.config.policies, **new_config.policies},
            public_keys={**self.config.public_keys, **new_config.public_keys},
            trusted_key_sources=new_config.trusted_key_sources or self.config.trusted_key_sources,
            performance_monitoring=new_config.performance_monitoring or self.config.performance_monitoring
        )
        
        self.config = merged_config
        self._validate_config()
    
    def add_public_key(self, key_id: str, public_key: bytes) -> None:
        """
        Add public key to configuration
        
        Args:
            key_id: Key identifier
            public_key: Public key bytes
        """
        if not isinstance(public_key, bytes) or len(public_key) != 32:
            raise VerificationError(
                'Public key must be 32 bytes',
                VerificationErrorCodes.INVALID_PUBLIC_KEY
            )
        
        self.config.public_keys[key_id] = bytes(public_key)  # Create copy
    
    def remove_public_key(self, key_id: str) -> None:
        """
        Remove public key from configuration
        
        Args:
            key_id: Key identifier to remove
        """
        self.config.public_keys.pop(key_id, None)
    
    def clear_nonce_cache(self) -> None:
        """Clear nonce cache (for replay protection)"""
        self.nonce_cache.clear()
    
    def _get_policy(self, policy_name: Optional[str]) -> VerificationPolicy:
        """
        Get verification policy by name
        
        Args:
            policy_name: Policy name (optional)
            
        Returns:
            VerificationPolicy: Verification policy
            
        Raises:
            VerificationError: If policy not found
        """
        name = policy_name or self.config.default_policy or 'standard'
        
        # Check config policies first
        if name in self.config.policies:
            return self.config.policies[name]
        
        # Check built-in policies
        policy = get_verification_policy(name)
        if policy:
            return policy
        
        raise VerificationError(
            f'Unknown verification policy: {name}',
            VerificationErrorCodes.UNKNOWN_POLICY,
            {'available_policies': list(self.config.policies.keys())}
        )
    
    async def _get_public_key(
        self,
        key_id: str,
        provided_key: Optional[bytes],
        skip_retrieval: bool
    ) -> bytes:
        """
        Get public key for verification
        
        Args:
            key_id: Key identifier
            provided_key: Provided public key (optional)
            skip_retrieval: Skip external key retrieval
            
        Returns:
            bytes: Public key
            
        Raises:
            VerificationError: If key not found
        """
        # Use provided key if available
        if provided_key:
            if len(provided_key) != 32:
                raise VerificationError(
                    'Public key must be 32 bytes',
                    VerificationErrorCodes.INVALID_PUBLIC_KEY
                )
            return provided_key
        
        # Check configuration
        if key_id in self.config.public_keys:
            return self.config.public_keys[key_id]
        
        # Skip key retrieval if requested
        if skip_retrieval:
            raise VerificationError(
                f'Public key not found for key ID: {key_id}',
                VerificationErrorCodes.PUBLIC_KEY_NOT_FOUND,
                {'key_id': key_id}
            )
        
        # Try key sources
        if self.config.trusted_key_sources:
            for source in self.config.trusted_key_sources:
                try:
                    key = await self._retrieve_key_from_source(source, key_id)
                    if key:
                        return key
                except Exception as error:
                    # Continue to next source
                    import logging
                    logger = logging.getLogger(__name__)
                    logger.warning(f"Key retrieval failed from source {source.name}: {error}")
        
        raise VerificationError(
            f'Public key not found for key ID: {key_id}',
            VerificationErrorCodes.PUBLIC_KEY_NOT_FOUND,
            {'key_id': key_id}
        )
    
    async def _retrieve_key_from_source(
        self,
        source: KeySource,
        key_id: str
    ) -> Optional[bytes]:
        """
        Retrieve key from external source
        
        Args:
            source: Key source
            key_id: Key identifier
            
        Returns:
            bytes or None: Retrieved key
        """
        try:
            return await source.retrieve_key(key_id)
        except Exception:
            return None
    
    async def _perform_verification(self, context: VerificationContext) -> VerificationResult:
        """
        Perform comprehensive signature verification
        
        Args:
            context: Verification context
            
        Returns:
            VerificationResult: Verification result
        """
        checks = {
            'format_valid': False,
            'cryptographic_valid': False,
            'timestamp_valid': False,
            'nonce_valid': False,
            'content_digest_valid': False,
            'component_coverage_valid': False,
            'custom_rules_valid': False
        }
        
        # Create diagnostics
        diagnostics = VerificationDiagnostics(
            signature_analysis={
                'algorithm': context.signature_data.params.alg.value,
                'key_id': context.signature_data.params.keyid,
                'created': context.signature_data.params.created,
                'age': int(time.time()) - context.signature_data.params.created,
                'nonce': context.signature_data.params.nonce,
                'covered_components': context.signature_data.covered_components
            },
            content_analysis={
                'has_content_digest': bool(context.signature_data.content_digest),
                'digest_algorithm': context.signature_data.content_digest.get('algorithm') if context.signature_data.content_digest else None,
                'content_size': get_content_size(context.message),
                'content_type': get_content_type(context.message)
            },
            policy_compliance={
                'policy_name': context.policy.name,
                'missing_required_components': [],
                'extra_components': [],
                'rule_results': []
            },
            security_analysis={
                'security_level': 'medium',
                'concerns': [],
                'recommendations': []
            }
        )
        
        # 1. Format validation
        checks['format_valid'] = self._validate_format(context)
        
        # 2. Cryptographic verification
        checks['cryptographic_valid'] = await self._verify_cryptographic_signature(context)
        
        # 3. Timestamp validation
        if context.policy.verify_timestamp:
            checks['timestamp_valid'] = self._validate_timestamp(context, diagnostics)
        else:
            checks['timestamp_valid'] = True
        
        # 4. Nonce validation
        if context.policy.verify_nonce:
            checks['nonce_valid'] = self._validate_nonce_format(context)
        else:
            checks['nonce_valid'] = True
        
        # 5. Content digest validation
        if context.policy.verify_content_digest:
            checks['content_digest_valid'] = await self._validate_content_digest(context)
        else:
            checks['content_digest_valid'] = True
        
        # 6. Component coverage validation
        checks['component_coverage_valid'] = self._validate_component_coverage(context, diagnostics)
        
        # 7. Custom rules validation
        if context.policy.custom_rules:
            checks['custom_rules_valid'] = await self._validate_custom_rules(context, diagnostics)
        else:
            checks['custom_rules_valid'] = True
        
        # Determine overall status
        all_checks_valid = all(checks.values())
        signature_valid = checks['format_valid'] and checks['cryptographic_valid']
        
        if all_checks_valid and signature_valid:
            status = VerificationStatus.VALID
        elif signature_valid:
            status = VerificationStatus.INVALID  # Signature valid but policy failed
        else:
            status = VerificationStatus.INVALID
        
        # Generate security analysis
        self._generate_security_analysis(checks, diagnostics)
        
        return VerificationResult(
            status=status,
            signature_valid=signature_valid,
            checks=checks,
            diagnostics=diagnostics,
            performance={'total_time': 0, 'step_timings': {}}  # Will be filled by caller
        )
    
    def _validate_format(self, context: VerificationContext) -> bool:
        """
        Validate signature format according to RFC 9421
        
        Args:
            context: Verification context
            
        Returns:
            bool: True if format is valid
        """
        try:
            params = context.signature_data.params
            
            # Check required parameters
            if not all([params.created, params.keyid, params.alg, params.nonce]):
                return False
            
            # Check algorithm is allowed
            if params.alg.value not in context.policy.allowed_algorithms:
                return False
            
            # Check covered components
            components = context.signature_data.covered_components
            if not components:
                return False
            
            return True
        except Exception:
            return False
    
    async def _verify_cryptographic_signature(self, context: VerificationContext) -> bool:
        """
        Verify cryptographic signature using Ed25519
        
        Args:
            context: Verification context
            
        Returns:
            bool: True if signature is cryptographically valid
        """
        try:
            # Reconstruct canonical message
            canonical_message = reconstruct_canonical_message(
                context.message,
                context.signature_data
            )
            
            # Convert hex signature to bytes
            signature_bytes = from_hex(context.signature_data.signature)
            
            # Verify signature
            return verify_signature(
                context.public_key,
                canonical_message,
                signature_bytes
            )
            
        except Exception as e:
            import logging
            logger = logging.getLogger(__name__)
            logger.debug(f"Cryptographic verification failed: {e}")
            return False
    
    def _validate_timestamp(
        self,
        context: VerificationContext,
        diagnostics: VerificationDiagnostics
    ) -> bool:
        """
        Validate timestamp according to policy
        
        Args:
            context: Verification context
            diagnostics: Diagnostics to update
            
        Returns:
            bool: True if timestamp is valid
        """
        timestamp = context.signature_data.params.created
        
        if not validate_timestamp(timestamp):
            return False
        
        # Check freshness if policy specifies max age
        if context.policy.max_timestamp_age is not None:
            if not check_timestamp_freshness(timestamp, context.policy.max_timestamp_age):
                return False
        
        return True
    
    def _validate_nonce_format(self, context: VerificationContext) -> bool:
        """
        Validate nonce format
        
        Args:
            context: Verification context
            
        Returns:
            bool: True if nonce format is valid
        """
        return validate_nonce(context.signature_data.params.nonce)
    
    async def _validate_content_digest(self, context: VerificationContext) -> bool:
        """
        Validate content digest if present
        
        Args:
            context: Verification context
            
        Returns:
            bool: True if content digest is valid
        """
        if not context.signature_data.content_digest:
            # No digest to validate - check if required
            if 'content-digest' in context.signature_data.covered_components:
                return False  # Required but missing
            return True  # Not required, no digest
        
        # Get content from message
        content = getattr(context.message, 'body', None)
        
        # Verify digest
        return verify_content_digest(content, context.signature_data.content_digest)
    
    def _validate_component_coverage(
        self,
        context: VerificationContext,
        diagnostics: VerificationDiagnostics
    ) -> bool:
        """
        Validate component coverage according to policy
        
        Args:
            context: Verification context
            diagnostics: Diagnostics to update
            
        Returns:
            bool: True if component coverage is valid
        """
        covered = context.signature_data.covered_components
        required = context.policy.required_components or []
        
        # Check for missing required components
        missing = [comp for comp in required if comp not in covered]
        diagnostics.policy_compliance['missing_required_components'] = missing
        
        # Check for extra components
        extra = [comp for comp in covered if comp not in required] if context.policy.require_all_headers else []
        diagnostics.policy_compliance['extra_components'] = extra
        
        # Validation passes if no missing required components
        return len(missing) == 0
    
    async def _validate_custom_rules(
        self,
        context: VerificationContext,
        diagnostics: VerificationDiagnostics
    ) -> bool:
        """
        Validate custom rules
        
        Args:
            context: Verification context
            diagnostics: Diagnostics to update
            
        Returns:
            bool: True if all custom rules pass
        """
        all_passed = True
        rule_results = []
        
        for rule in context.policy.custom_rules or []:
            try:
                # Call rule validation function
                result = rule.validate(context)
                
                # Handle async results
                if hasattr(result, '__await__'):
                    result = await result
                
                rule_results.append(result)
                if not result.passed:
                    all_passed = False
                    
            except Exception as e:
                # Rule validation failed
                error_result = VerificationRuleResult(
                    passed=False,
                    message=f'Rule validation error: {e}',
                    details={'rule_name': rule.name, 'error': str(e)}
                )
                rule_results.append(error_result)
                all_passed = False
        
        diagnostics.policy_compliance['rule_results'] = [
            {
                'passed': r.passed,
                'message': r.message,
                'details': r.details
            } for r in rule_results
        ]
        
        return all_passed
    
    def _generate_security_analysis(
        self,
        checks: Dict[str, bool],
        diagnostics: VerificationDiagnostics
    ) -> None:
        """
        Generate security analysis based on verification results
        
        Args:
            checks: Verification check results
            diagnostics: Diagnostics to update
        """
        concerns = []
        recommendations = []
        
        # Analyze individual checks
        if not checks['cryptographic_valid']:
            concerns.append('Cryptographic signature verification failed')
            recommendations.append('Verify signature generation and key management')
        
        if not checks['timestamp_valid']:
            concerns.append('Timestamp validation failed')
            recommendations.append('Check system clocks and timestamp policies')
        
        if not checks['nonce_valid']:
            concerns.append('Nonce validation failed')
            recommendations.append('Ensure proper nonce generation and format')
        
        if not checks['content_digest_valid']:
            concerns.append('Content digest validation failed')
            recommendations.append('Verify content integrity and digest calculation')
        
        if not checks['component_coverage_valid']:
            concerns.append('Component coverage requirements not met')
            recommendations.append('Review signature component coverage policy')
        
        # Determine security level
        valid_checks = sum(1 for passed in checks.values() if passed)
        total_checks = len(checks)
        
        if valid_checks == total_checks:
            security_level = 'high'
        elif valid_checks >= total_checks * 0.7:
            security_level = 'medium'
        else:
            security_level = 'low'
        
        diagnostics.security_analysis.update({
            'security_level': security_level,
            'concerns': concerns,
            'recommendations': recommendations
        })
    
    def _create_error_result(
        self,
        error: VerificationError,
        total_time: float,
        step_timings: Dict[str, float]
    ) -> VerificationResult:
        """
        Create error verification result
        
        Args:
            error: Verification error
            total_time: Total verification time
            step_timings: Step timing breakdown
            
        Returns:
            VerificationResult: Error result
        """
        return VerificationResult(
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
            performance={
                'total_time': total_time,
                'step_timings': step_timings
            },
            error={
                'code': error.code,
                'message': error.message,
                'details': error.details
            }
        )
    
    def _validate_config(self) -> None:
        """
        Validate verification configuration
        
        Raises:
            VerificationError: If configuration is invalid
        """
        if not isinstance(self.config.policies, dict):
            raise VerificationError(
                'Policies must be a dictionary',
                VerificationErrorCodes.INVALID_CONFIG
            )
        
        if not isinstance(self.config.public_keys, dict):
            raise VerificationError(
                'Public keys must be a dictionary',
                VerificationErrorCodes.INVALID_CONFIG
            )
        
        # Validate public keys
        for key_id, key_bytes in self.config.public_keys.items():
            if not isinstance(key_bytes, bytes) or len(key_bytes) != 32:
                raise VerificationError(
                    f'Invalid public key for {key_id}: must be 32 bytes',
                    VerificationErrorCodes.INVALID_PUBLIC_KEY
                )
    
    def _copy_config(self, config: VerificationConfig) -> VerificationConfig:
        """
        Create a defensive copy of verification configuration
        
        Args:
            config: Original configuration
            
        Returns:
            VerificationConfig: Copied configuration
        """
        return VerificationConfig(
            default_policy=config.default_policy,
            policies=config.policies.copy(),
            public_keys={k: bytes(v) for k, v in config.public_keys.items()},
            trusted_key_sources=config.trusted_key_sources.copy() if config.trusted_key_sources else [],
            performance_monitoring=config.performance_monitoring.copy() if config.performance_monitoring else {}
        )


def create_verifier(config: VerificationConfig) -> RFC9421Verifier:
    """
    Create a new RFC 9421 verifier
    
    Args:
        config: Verification configuration
        
    Returns:
        RFC9421Verifier: Configured verifier instance
    """
    return RFC9421Verifier(config)


async def verify_signature(
    message: Union[SignableRequest, VerifiableResponse],
    headers: Dict[str, str],
    config: VerificationConfig,
    policy: Optional[str] = None,
    public_key: Optional[bytes] = None
) -> VerificationResult:
    """
    Verify a signature with the given configuration
    
    Args:
        message: Message to verify
        headers: HTTP headers containing signature
        config: Verification configuration
        policy: Verification policy name
        public_key: Public key for verification
        
    Returns:
        VerificationResult: Verification result
    """
    verifier = create_verifier(config)
    return await verifier.verify(message, headers, policy, public_key)


async def verify_response(
    response: VerifiableResponse,
    headers: Dict[str, str],
    config: VerificationConfig,
    policy: Optional[str] = None
) -> VerificationResult:
    """
    Verify a response signature
    
    Args:
        response: Response to verify
        headers: HTTP headers containing signature
        config: Verification configuration
        policy: Verification policy name
        
    Returns:
        VerificationResult: Verification result
    """
    return await verify_signature(response, headers, config, policy)


async def verify_request(
    request: SignableRequest,
    headers: Dict[str, str],
    config: VerificationConfig,
    policy: Optional[str] = None
) -> VerificationResult:
    """
    Verify a request signature
    
    Args:
        request: Request to verify
        headers: HTTP headers containing signature
        config: Verification configuration
        policy: Verification policy name
        
    Returns:
        VerificationResult: Verification result
    """
    return await verify_signature(request, headers, config, policy)