"""
Signature inspection and debugging utilities

This module provides debugging and analysis tools for RFC 9421 HTTP Message Signatures,
including format inspection, component analysis, and diagnostic reporting.
"""

import re
from typing import Dict, List, Optional, Any

from .types import (
    SignatureInspector,
    SignatureFormatAnalysis,
    ComponentAnalysis,
    ParameterValidation,
    FormatIssue,
    ComponentIssue,
    ComponentSecurityAssessment,
    ExtractedSignatureData,
    VerificationResult,
    VerificationError,
    VerificationErrorCodes,
)
from .utils import (
    find_header_case_insensitive,
    parse_signature_input,
    validate_component_name,
    is_pseudo_component,
)
from ..signing.types import SignatureParams
from ..signing.utils import validate_timestamp, validate_nonce, validate_header_name


class RFC9421Inspector(SignatureInspector):
    """
    RFC 9421 signature inspector for debugging and analysis
    
    This class provides comprehensive signature inspection capabilities
    for debugging signature verification issues and analyzing security.
    """
    
    def inspect_format(self, headers: Dict[str, str]) -> SignatureFormatAnalysis:
        """
        Inspect signature format and structure
        
        Args:
            headers: HTTP headers to inspect
            
        Returns:
            SignatureFormatAnalysis: Format analysis result
        """
        issues: List[FormatIssue] = []
        signature_headers: List[str] = []
        signature_ids: List[str] = []
        
        # Find signature-related headers (case-insensitive)
        found_headers = self._find_signature_headers(headers)
        signature_headers.extend(found_headers.keys())
        
        # Check for required headers
        if 'signature-input' not in found_headers:
            issues.append(FormatIssue(
                severity='error',
                code='MISSING_SIGNATURE_INPUT',
                message='Signature-Input header is required',
                component='signature-input'
            ))
        
        if 'signature' not in found_headers:
            issues.append(FormatIssue(
                severity='error',
                code='MISSING_SIGNATURE',
                message='Signature header is required',
                component='signature'
            ))
        
        # Analyze signature-input header
        if 'signature-input' in found_headers:
            try:
                parsed = parse_signature_input(found_headers['signature-input'])
                signature_ids.append(parsed['signature_name'])
                
                # Validate signature input format
                self._validate_signature_input_format(found_headers['signature-input'], issues)
                
                # Validate parameters
                self._validate_signature_parameters(parsed['params'], issues)
                
                # Validate covered components
                self._validate_covered_components(parsed['covered_components'], issues)
                
            except VerificationError as error:
                issues.append(FormatIssue(
                    severity='error',
                    code='INVALID_SIGNATURE_INPUT_FORMAT',
                    message=f'Failed to parse signature-input: {error.message}',
                    component='signature-input'
                ))
        
        # Analyze signature header
        if 'signature' in found_headers:
            self._validate_signature_header_format(found_headers['signature'], issues)
        
        # Check for content-digest if present
        if 'content-digest' in found_headers:
            self._validate_content_digest_format(found_headers['content-digest'], issues)
        
        is_valid_rfc9421 = len([issue for issue in issues if issue.severity == 'error']) == 0
        
        return SignatureFormatAnalysis(
            is_valid_rfc9421=is_valid_rfc9421,
            issues=issues,
            signature_headers=signature_headers,
            signature_ids=signature_ids
        )
    
    def analyze_components(self, signature_data: ExtractedSignatureData) -> ComponentAnalysis:
        """
        Analyze signature components
        
        Args:
            signature_data: Extracted signature data
            
        Returns:
            ComponentAnalysis: Component analysis result
        """
        valid_components: List[str] = []
        invalid_components: List[ComponentIssue] = []
        missing_components: List[str] = []
        
        # Analyze each covered component
        for component in signature_data.covered_components:
            analysis = self._analyze_component(component)
            
            if analysis['valid']:
                valid_components.append(component)
            else:
                invalid_components.append(ComponentIssue(
                    component=component,
                    type='format',
                    message=analysis.get('message', 'Invalid component format')
                ))
        
        # Check for recommended components
        recommended_components = ['@method', '@target-uri']
        for recommended in recommended_components:
            if recommended not in signature_data.covered_components:
                missing_components.append(recommended)
        
        # Generate security assessment
        security_assessment = self._assess_component_security(signature_data.covered_components)
        
        return ComponentAnalysis(
            valid_components=valid_components,
            invalid_components=invalid_components,
            missing_components=missing_components,
            security_assessment=security_assessment
        )
    
    def validate_parameters(self, params: SignatureParams) -> ParameterValidation:
        """
        Validate signature parameters
        
        Args:
            params: Signature parameters to validate
            
        Returns:
            ParameterValidation: Parameter validation result
        """
        validation = ParameterValidation(
            all_valid=True,
            parameters={
                'created': {'valid': True},
                'keyid': {'valid': True},
                'alg': {'valid': True},
                'nonce': {'valid': True}
            },
            insights=[]
        )
        
        # Validate created timestamp
        if not validate_timestamp(params.created):
            validation.parameters['created'] = {
                'valid': False,
                'message': 'Invalid timestamp format or value'
            }
            validation.all_valid = False
        else:
            import time
            now = int(time.time())
            age = now - params.created
            
            if age > 3600:
                validation.insights.append(f'Timestamp is {age // 60} minutes old')
            
            if age < 0:
                validation.insights.append('Timestamp is from the future (possible clock skew)')
        
        # Validate key ID
        if not params.keyid or not isinstance(params.keyid, str) or not params.keyid.strip():
            validation.parameters['keyid'] = {
                'valid': False,
                'message': 'Key ID must be a non-empty string'
            }
            validation.all_valid = False
        
        # Validate algorithm
        if params.alg.value != 'ed25519':
            validation.parameters['alg'] = {
                'valid': False,
                'message': f'Unsupported algorithm: {params.alg.value}'
            }
            validation.all_valid = False
        
        # Validate nonce
        if not validate_nonce(params.nonce):
            validation.parameters['nonce'] = {
                'valid': False,
                'message': 'Invalid nonce format (should be UUID v4)'
            }
            validation.all_valid = False
        
        return validation
    
    def generate_diagnostic_report(self, result: VerificationResult) -> str:
        """
        Generate diagnostic report
        
        Args:
            result: Verification result to analyze
            
        Returns:
            str: Formatted diagnostic report
        """
        lines = []
        
        lines.append('=== RFC 9421 Signature Verification Report ===')
        lines.append('')
        
        # Overall status
        lines.append(f'Overall Status: {result.status.value.upper()}')
        lines.append(f'Signature Valid: {"YES" if result.signature_valid else "NO"}')
        lines.append('')
        
        # Individual checks
        lines.append('=== Individual Checks ===')
        for check, passed in result.checks.items():
            status = '✓' if passed else '✗'
            label = check.replace('_', ' ').title()
            lines.append(f'{status} {label}')
        lines.append('')
        
        # Signature analysis
        sig = result.diagnostics.signature_analysis
        lines.append('=== Signature Analysis ===')
        lines.append(f'Algorithm: {sig.get("algorithm", "N/A")}')
        lines.append(f'Key ID: {sig.get("key_id", "N/A")}')
        
        if sig.get('created'):
            import time
            created_time = time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime(sig['created']))
            lines.append(f'Created: {created_time}')
            lines.append(f'Age: {sig.get("age", 0)} seconds')
        
        lines.append(f'Nonce: {sig.get("nonce", "N/A")}')
        components = sig.get('covered_components', [])
        lines.append(f'Covered Components: {", ".join(components) if components else "None"}')
        lines.append('')
        
        # Content analysis
        content = result.diagnostics.content_analysis
        lines.append('=== Content Analysis ===')
        lines.append(f'Has Content Digest: {"YES" if content.get("has_content_digest") else "NO"}')
        if content.get('digest_algorithm'):
            lines.append(f'Digest Algorithm: {content["digest_algorithm"]}')
        lines.append(f'Content Size: {content.get("content_size", 0)} bytes')
        if content.get('content_type'):
            lines.append(f'Content Type: {content["content_type"]}')
        lines.append('')
        
        # Policy compliance
        policy = result.diagnostics.policy_compliance
        lines.append('=== Policy Compliance ===')
        lines.append(f'Policy: {policy.get("policy_name", "N/A")}')
        
        missing = policy.get('missing_required_components', [])
        if missing:
            lines.append(f'Missing Required Components: {", ".join(missing)}')
        
        extra = policy.get('extra_components', [])
        if extra:
            lines.append(f'Extra Components: {", ".join(extra)}')
        
        # Custom rule results
        rule_results = policy.get('rule_results', [])
        if rule_results:
            lines.append('')
            lines.append('=== Custom Rule Results ===')
            for rule_result in rule_results:
                status = '✓' if rule_result.get('passed') else '✗'
                message = rule_result.get('message', 'Rule validation')
                lines.append(f'{status} {message}')
        
        # Security analysis
        security = result.diagnostics.security_analysis
        lines.append('')
        lines.append('=== Security Analysis ===')
        lines.append(f'Security Level: {security.get("security_level", "unknown").upper()}')
        
        concerns = security.get('concerns', [])
        if concerns:
            lines.append('')
            lines.append('Security Concerns:')
            for concern in concerns:
                lines.append(f'  - {concern}')
        
        recommendations = security.get('recommendations', [])
        if recommendations:
            lines.append('')
            lines.append('Recommendations:')
            for recommendation in recommendations:
                lines.append(f'  - {recommendation}')
        
        # Performance
        lines.append('')
        lines.append('=== Performance ===')
        total_time = result.performance.get('total_time', 0)
        lines.append(f'Total Time: {total_time:.2f}ms')
        
        step_timings = result.performance.get('step_timings', {})
        if step_timings:
            lines.append('Step Timings:')
            for step, timing in step_timings.items():
                lines.append(f'  - {step}: {timing:.2f}ms')
        
        # Error details
        if result.error:
            lines.append('')
            lines.append('=== Error Details ===')
            lines.append(f'Code: {result.error["code"]}')
            lines.append(f'Message: {result.error["message"]}')
            
            details = result.error.get('details', {})
            if details:
                lines.append('Details:')
                for key, value in details.items():
                    lines.append(f'  - {key}: {value}')
        
        return '\n'.join(lines)
    
    def _find_signature_headers(self, headers: Dict[str, str]) -> Dict[str, str]:
        """
        Find signature-related headers (case-insensitive)
        
        Args:
            headers: HTTP headers
            
        Returns:
            dict: Found signature headers
        """
        found = {}
        target_headers = ['signature-input', 'signature', 'content-digest']
        
        for target in target_headers:
            value = find_header_case_insensitive(headers, target)
            if value:
                found[target] = value
        
        return found
    
    def _validate_signature_input_format(self, signature_input: str, issues: List[FormatIssue]) -> None:
        """
        Validate signature input format
        
        Args:
            signature_input: Signature input header value
            issues: List to append issues to
        """
        # Check basic format: sig1=("@method" "@target-uri");created=123;keyid="key"
        if not all(char in signature_input for char in ['=', '(', ')']):
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_FORMAT',
                message='Signature-Input header format is invalid',
                component='signature-input'
            ))
            return
        
        # Check for required parameters
        required_params = ['created', 'keyid', 'alg']
        for param in required_params:
            if f'{param}=' not in signature_input:
                issues.append(FormatIssue(
                    severity='error',
                    code='MISSING_PARAMETER',
                    message=f'Missing required parameter: {param}',
                    component='signature-input'
                ))
        
        # Check component list format
        component_match = re.search(r'\(([^)]+)\)', signature_input)
        if component_match:
            component_list = component_match.group(1)
            if '"' not in component_list:
                issues.append(FormatIssue(
                    severity='warning',
                    code='UNQUOTED_COMPONENTS',
                    message='Components should be quoted according to RFC 9421',
                    component='signature-input'
                ))
    
    def _validate_signature_parameters(self, params: SignatureParams, issues: List[FormatIssue]) -> None:
        """
        Validate signature parameters
        
        Args:
            params: Signature parameters
            issues: List to append issues to
        """
        # Check timestamp
        if not validate_timestamp(params.created):
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_TIMESTAMP',
                message='Invalid created timestamp',
                component='created'
            ))
        
        # Check nonce
        if not validate_nonce(params.nonce):
            issues.append(FormatIssue(
                severity='warning',
                code='INVALID_NONCE_FORMAT',
                message='Nonce does not follow UUID v4 format',
                component='nonce'
            ))
        
        # Check algorithm
        if params.alg.value != 'ed25519':
            issues.append(FormatIssue(
                severity='warning',
                code='UNSUPPORTED_ALGORITHM',
                message=f'Algorithm {params.alg.value} is not ed25519',
                component='alg'
            ))
    
    def _validate_covered_components(self, components: List[str], issues: List[FormatIssue]) -> None:
        """
        Validate covered components
        
        Args:
            components: List of covered components
            issues: List to append issues to
        """
        if not components:
            issues.append(FormatIssue(
                severity='error',
                code='NO_COMPONENTS',
                message='No signature components specified',
                component='components'
            ))
            return
        
        for component in components:
            if is_pseudo_component(component):
                # Pseudo-component validation
                allowed_pseudo = ['@method', '@target-uri', '@authority', '@scheme', '@request-target']
                if component not in allowed_pseudo:
                    issues.append(FormatIssue(
                        severity='warning',
                        code='UNKNOWN_PSEUDO_COMPONENT',
                        message=f'Unknown pseudo-component: {component}',
                        component='components'
                    ))
            else:
                # HTTP header validation
                if not validate_header_name(component):
                    issues.append(FormatIssue(
                        severity='error',
                        code='INVALID_HEADER_NAME',
                        message=f'Invalid header name: {component}',
                        component='components'
                    ))
    
    def _validate_signature_header_format(self, signature: str, issues: List[FormatIssue]) -> None:
        """
        Validate signature header format
        
        Args:
            signature: Signature header value
            issues: List to append issues to
        """
        # Check format: sig1=:hexsignature:
        signature_pattern = re.compile(r'^[^=]+=:[0-9a-fA-F]+:$')
        if not signature_pattern.match(signature):
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_SIGNATURE_FORMAT',
                message='Signature header format is invalid (should be name=:hex:)',
                component='signature'
            ))
        
        # Check signature length for Ed25519 (64 hex chars = 32 bytes)
        hex_match = re.search(r':([0-9a-fA-F]+):', signature)
        if hex_match:
            hex_signature = hex_match.group(1)
            if len(hex_signature) != 128:  # 64 bytes * 2 hex chars
                issues.append(FormatIssue(
                    severity='warning',
                    code='UNEXPECTED_SIGNATURE_LENGTH',
                    message=f'Signature length is {len(hex_signature)} hex chars, expected 128 for Ed25519',
                    component='signature'
                ))
    
    def _validate_content_digest_format(self, content_digest: str, issues: List[FormatIssue]) -> None:
        """
        Validate content digest format
        
        Args:
            content_digest: Content digest header value
            issues: List to append issues to
        """
        # Check format: algorithm=:base64digest:
        digest_pattern = re.compile(r'^[^=]+=:[A-Za-z0-9+/]+=*:$')
        if not digest_pattern.match(content_digest):
            issues.append(FormatIssue(
                severity='error',
                code='INVALID_CONTENT_DIGEST_FORMAT',
                message='Content-Digest header format is invalid',
                component='content-digest'
            ))
        
        # Check for supported algorithms
        if content_digest.startswith('sha-256=:') or content_digest.startswith('sha-512=:'):
            pass  # Supported
        else:
            algorithm = content_digest.split('=:', 1)[0] if '=:' in content_digest else 'unknown'
            issues.append(FormatIssue(
                severity='warning',
                code='UNSUPPORTED_DIGEST_ALGORITHM',
                message=f'Digest algorithm may not be supported: {algorithm}',
                component='content-digest'
            ))
    
    def _analyze_component(self, component: str) -> Dict[str, Any]:
        """
        Analyze individual component
        
        Args:
            component: Component name to analyze
            
        Returns:
            dict: Analysis result
        """
        if is_pseudo_component(component):
            allowed_pseudo = ['@method', '@target-uri', '@authority', '@scheme', '@request-target']
            if component in allowed_pseudo:
                return {'valid': True}
            else:
                return {'valid': False, 'message': f'Unknown pseudo-component: {component}'}
        else:
            if validate_header_name(component):
                return {'valid': True}
            else:
                return {'valid': False, 'message': f'Invalid header name: {component}'}
    
    def _assess_component_security(self, components: List[str]) -> ComponentSecurityAssessment:
        """
        Assess component security
        
        Args:
            components: List of covered components
            
        Returns:
            ComponentSecurityAssessment: Security assessment
        """
        strengths = []
        weaknesses = []
        score = 0
        
        # Essential components
        if '@method' in components:
            strengths.append('HTTP method is covered')
            score += 20
        else:
            weaknesses.append('HTTP method not covered')
        
        if '@target-uri' in components:
            strengths.append('Target URI is covered')
            score += 20
        else:
            weaknesses.append('Target URI not covered')
        
        # Content integrity
        if 'content-digest' in components:
            strengths.append('Content integrity protected')
            score += 30
        else:
            weaknesses.append('Content integrity not protected')
        
        # Headers coverage
        header_components = [c for c in components if not is_pseudo_component(c) and c != 'content-digest']
        if len(header_components) >= 2:
            strengths.append(f'Good header coverage ({len(header_components)} headers)')
            score += 20
        elif len(header_components) == 1:
            score += 10
        else:
            weaknesses.append('Limited header coverage')
        
        # Security-sensitive headers
        security_headers = ['authorization', 'content-type', 'date', 'host']
        covered_security_headers = [h for h in header_components if h.lower() in security_headers]
        if covered_security_headers:
            strengths.append(f'Security headers covered: {", ".join(covered_security_headers)}')
            score += len(covered_security_headers) * 5
        
        # Determine level
        if score >= 80:
            level = 'high'
        elif score >= 50:
            level = 'medium'
        else:
            level = 'low'
        
        return ComponentSecurityAssessment(
            level=level,
            score=min(score, 100),
            strengths=strengths,
            weaknesses=weaknesses
        )


def create_inspector() -> RFC9421Inspector:
    """
    Create a new RFC 9421 inspector
    
    Returns:
        RFC9421Inspector: Inspector instance
    """
    return RFC9421Inspector()


def validate_signature_format(headers: Dict[str, str]) -> bool:
    """
    Quick signature format validation
    
    Args:
        headers: HTTP headers to validate
        
    Returns:
        bool: True if format is valid
    """
    inspector = create_inspector()
    analysis = inspector.inspect_format(headers)
    return analysis.is_valid_rfc9421


def quick_diagnostic(headers: Dict[str, str]) -> str:
    """
    Generate quick diagnostic report for headers
    
    Args:
        headers: HTTP headers to analyze
        
    Returns:
        str: Quick diagnostic report
    """
    inspector = create_inspector()
    analysis = inspector.inspect_format(headers)
    
    lines = []
    lines.append('=== Quick Signature Diagnostic ===')
    lines.append(f'RFC 9421 Compliant: {"YES" if analysis.is_valid_rfc9421 else "NO"}')
    lines.append(f'Signature Headers Found: {", ".join(analysis.signature_headers)}')
    lines.append(f'Signature IDs: {", ".join(analysis.signature_ids)}')
    
    if analysis.issues:
        lines.append('')
        lines.append('Issues Found:')
        for issue in analysis.issues:
            severity_icon = '❌' if issue.severity == 'error' else '⚠️' if issue.severity == 'warning' else 'ℹ️'
            lines.append(f'  {severity_icon} {issue.message}')
    
    return '\n'.join(lines)


def analyze_signature_security(signature_data: ExtractedSignatureData) -> Dict[str, Any]:
    """
    Analyze signature security characteristics
    
    Args:
        signature_data: Extracted signature data
        
    Returns:
        dict: Security analysis
    """
    inspector = create_inspector()
    component_analysis = inspector.analyze_components(signature_data)
    param_validation = inspector.validate_parameters(signature_data.params)
    
    return {
        'component_security': component_analysis.security_assessment.__dict__,
        'parameter_validity': param_validation.all_valid,
        'parameter_insights': param_validation.insights,
        'valid_components': component_analysis.valid_components,
        'invalid_components': [issue.__dict__ for issue in component_analysis.invalid_components],
        'missing_recommended': component_analysis.missing_components
    }