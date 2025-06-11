"""
Example usage of DataFold Python SDK signature verification utilities

This example demonstrates how to use the verification functionality to validate
RFC 9421 HTTP Message Signatures in various scenarios.
"""

import asyncio
import time
from typing import Dict, Any

from datafold_sdk.verification import (
    VerificationConfig,
    VerificationPolicy,
    RFC9421Verifier,
    RFC9421Inspector,
    VerificationResult,
    VerificationStatus,
    VerifiableResponse,
    create_verifier,
    verify_signature,
    verify_response,
    VERIFICATION_POLICIES,
    STRICT_VERIFICATION_POLICY,
    STANDARD_VERIFICATION_POLICY,
    LENIENT_VERIFICATION_POLICY,
)
from datafold_sdk.verification.middleware import (
    ResponseVerificationConfig,
    RequestVerificationConfig,
    create_response_verification_middleware,
    create_request_verification_middleware,
    create_batch_verifier,
)
from datafold_sdk.verification.utils import (
    extract_signature_data,
    validate_signature_format,
    quick_diagnostic,
)
from datafold_sdk.signing.types import SignableRequest, HttpMethod
from datafold_sdk.crypto.ed25519 import generate_key_pair


def create_sample_public_keys() -> Dict[str, bytes]:
    """Create sample public keys for demonstration"""
    key_pairs = {
        'server-key-1': generate_key_pair(),
        'server-key-2': generate_key_pair(),
        'client-key-1': generate_key_pair(),
    }
    
    return {key_id: kp.public_key for key_id, kp in key_pairs.items()}


def create_sample_signed_headers() -> Dict[str, str]:
    """Create sample signed headers for demonstration"""
    return {
        'signature-input': 'sig1=("@method" "@target-uri" "content-type" "date");created=1640995200;keyid="server-key-1";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:2ba8b1a0fc3d21d8c66e0e9fbc8b4bb6d2e4c5b9a9a1c7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6:',
        'content-type': 'application/json',
        'date': 'Sat, 01 Jan 2022 00:00:00 GMT',
        'content-digest': 'sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:'
    }


async def basic_verification_example():
    """Demonstrate basic signature verification"""
    print("=== Basic Signature Verification Example ===")
    
    # Create verification configuration
    public_keys = create_sample_public_keys()
    config = VerificationConfig(
        default_policy='standard',
        public_keys=public_keys
    )
    
    # Create verifier
    verifier = create_verifier(config)
    
    # Create sample response to verify
    response = VerifiableResponse(
        status=200,
        headers=create_sample_signed_headers(),
        body='{"message": "Hello, world!"}',
        url='https://api.example.com/data',
        method='GET'
    )
    
    # Extract signature headers
    signature_headers = {k: v for k, v in response.headers.items() 
                        if k.lower() in ['signature-input', 'signature', 'content-digest']}
    
    try:
        # Verify the signature (will fail crypto verification with mock data)
        result = await verifier.verify(
            response,
            signature_headers,
            policy='lenient'  # Use lenient for demo
        )
        
        print(f"Verification Status: {result.status.value}")
        print(f"Signature Valid: {result.signature_valid}")
        print(f"Format Valid: {result.checks['format_valid']}")
        print(f"Total Time: {result.performance['total_time']:.2f}ms")
        
        if result.error:
            print(f"Error: {result.error['message']}")
        
    except Exception as e:
        print(f"Verification failed: {e}")
    
    print()


async def policy_comparison_example():
    """Demonstrate different verification policies"""
    print("=== Verification Policy Comparison ===")
    
    public_keys = create_sample_public_keys()
    config = VerificationConfig(public_keys=public_keys)
    verifier = create_verifier(config)
    
    # Create test request
    request = SignableRequest(
        method=HttpMethod.POST,
        url='https://api.example.com/submit',
        headers=create_sample_signed_headers(),
        body='{"data": "test"}'
    )
    
    signature_headers = {k: v for k, v in request.headers.items() 
                        if k.lower() in ['signature-input', 'signature', 'content-digest']}
    
    policies = ['strict', 'standard', 'lenient', 'legacy']
    
    for policy_name in policies:
        try:
            result = await verifier.verify(request, signature_headers, policy=policy_name)
            print(f"{policy_name.upper()} Policy:")
            print(f"  Status: {result.status.value}")
            print(f"  Required Components Check: {result.checks['component_coverage_valid']}")
            print(f"  Timestamp Check: {result.checks['timestamp_valid']}")
            print(f"  Nonce Check: {result.checks['nonce_valid']}")
            print(f"  Content Digest Check: {result.checks['content_digest_valid']}")
            print()
        except Exception as e:
            print(f"{policy_name.upper()} Policy: Failed - {e}")
            print()


async def signature_inspection_example():
    """Demonstrate signature inspection and debugging"""
    print("=== Signature Inspection Example ===")
    
    headers = create_sample_signed_headers()
    inspector = RFC9421Inspector()
    
    # Inspect signature format
    format_analysis = inspector.inspect_format(headers)
    print(f"RFC 9421 Compliant: {format_analysis.is_valid_rfc9421}")
    print(f"Signature IDs Found: {', '.join(format_analysis.signature_ids)}")
    print(f"Issues Found: {len(format_analysis.issues)}")
    
    for issue in format_analysis.issues:
        print(f"  - {issue.severity.upper()}: {issue.message}")
    
    # Extract and analyze signature data
    try:
        signature_data = extract_signature_data(headers)
        
        # Analyze components
        component_analysis = inspector.analyze_components(signature_data)
        print(f"\nValid Components: {', '.join(component_analysis.valid_components)}")
        print(f"Security Level: {component_analysis.security_assessment.level}")
        print(f"Security Score: {component_analysis.security_assessment.score}/100")
        
        # Validate parameters
        param_validation = inspector.validate_parameters(signature_data.params)
        print(f"\nAll Parameters Valid: {param_validation.all_valid}")
        
        for param, validation in param_validation.parameters.items():
            status = "✓" if validation['valid'] else "✗"
            print(f"  {status} {param}")
        
        if param_validation.insights:
            print("Insights:")
            for insight in param_validation.insights:
                print(f"  - {insight}")
    
    except Exception as e:
        print(f"Signature data extraction failed: {e}")
    
    print()


async def batch_verification_example():
    """Demonstrate batch verification"""
    print("=== Batch Verification Example ===")
    
    public_keys = create_sample_public_keys()
    config = VerificationConfig(public_keys=public_keys)
    batch_verifier = create_batch_verifier(config)
    
    # Create multiple verification items
    headers = create_sample_signed_headers()
    items = []
    
    for i in range(3):
        request = SignableRequest(
            method=HttpMethod.GET,
            url=f'https://api.example.com/item/{i}',
            headers=headers,
            body=None
        )
        
        items.append({
            'message': request,
            'headers': headers,
            'policy': 'lenient'
        })
    
    # Perform batch verification
    results = await batch_verifier.verify_batch(items)
    
    print(f"Verified {len(results)} items")
    
    # Get batch statistics
    stats = batch_verifier.get_batch_stats(results)
    print(f"Success Rate: {stats['success_rate']:.1%}")
    print(f"Average Time: {stats['average_time']:.2f}ms")
    print(f"Total Time: {stats['total_time']:.2f}ms")
    
    # Show individual results
    for i, result in enumerate(results):
        print(f"Item {i}: {result.status.value}")
    
    print()


async def middleware_example():
    """Demonstrate verification middleware"""
    print("=== Verification Middleware Example ===")
    
    public_keys = create_sample_public_keys()
    verification_config = VerificationConfig(public_keys=public_keys)
    
    # Create response verification middleware
    response_config = ResponseVerificationConfig(
        verification_config=verification_config,
        default_policy='standard',
        throw_on_failure=False
    )
    
    response_middleware = create_response_verification_middleware(response_config)
    
    # Create mock response object
    class MockResponse:
        def __init__(self):
            self.status = 200
            self.url = 'https://api.example.com/data'
            self.headers = create_sample_signed_headers()
            self.text = '{"result": "success"}'
    
    response = MockResponse()
    
    try:
        # Apply middleware
        processed_response = await response_middleware(response)
        
        print("Response verification middleware applied successfully")
        
        # Check if verification result was added
        if hasattr(processed_response, '_verification_result'):
            result = processed_response._verification_result
            print(f"Verification Status: {result.status.value}")
        
    except Exception as e:
        print(f"Middleware failed: {e}")
    
    print()


def diagnostic_tools_example():
    """Demonstrate diagnostic tools"""
    print("=== Diagnostic Tools Example ===")
    
    headers = create_sample_signed_headers()
    
    # Quick format validation
    is_valid, issues = validate_signature_format(headers)
    print(f"Quick Format Check: {'PASS' if is_valid else 'FAIL'}")
    
    if issues:
        print("Issues found:")
        for issue in issues:
            print(f"  - {issue.severity.upper()}: {issue.message}")
    
    # Quick diagnostic
    diagnostic = quick_diagnostic(headers)
    print("\nQuick Diagnostic Report:")
    print(diagnostic)
    
    print()


async def custom_policy_example():
    """Demonstrate creating custom verification policies"""
    print("=== Custom Verification Policy Example ===")
    
    from datafold_sdk.verification.policies import (
        create_verification_policy,
        VerificationRules,
        register_verification_policy
    )
    
    # Create custom policy with specific rules
    custom_policy = create_verification_policy(
        name='api-gateway',
        description='Policy for API Gateway responses',
        verify_timestamp=True,
        max_timestamp_age=600,  # 10 minutes
        verify_nonce=True,
        verify_content_digest=True,
        required_components=['@method', '@target-uri', 'content-type', 'date'],
        custom_rules=[
            VerificationRules.timestamp_freshness(600),
            VerificationRules.required_headers(['content-type', 'date']),
            VerificationRules.content_type_consistency(),
        ]
    )
    
    # Register the custom policy
    register_verification_policy(custom_policy)
    
    print(f"Created custom policy: {custom_policy.name}")
    print(f"Description: {custom_policy.description}")
    print(f"Required components: {', '.join(custom_policy.required_components)}")
    print(f"Custom rules: {len(custom_policy.custom_rules)}")
    
    # Use the custom policy
    public_keys = create_sample_public_keys()
    config = VerificationConfig(
        default_policy='api-gateway',
        public_keys=public_keys
    )
    
    verifier = create_verifier(config)
    
    request = SignableRequest(
        method=HttpMethod.GET,
        url='https://gateway.example.com/api/v1/data',
        headers=create_sample_signed_headers(),
        body=None
    )
    
    signature_headers = {k: v for k, v in request.headers.items() 
                        if k.lower() in ['signature-input', 'signature', 'content-digest']}
    
    try:
        result = await verifier.verify(request, signature_headers)
        print(f"\nCustom policy verification: {result.status.value}")
        print(f"Custom rules check: {result.checks['custom_rules_valid']}")
    except Exception as e:
        print(f"Custom policy verification failed: {e}")
    
    print()


async def main():
    """Run all examples"""
    print("DataFold Python SDK Signature Verification Examples")
    print("=" * 60)
    print()
    
    await basic_verification_example()
    await policy_comparison_example()
    await signature_inspection_example()
    await batch_verification_example()
    await middleware_example()
    diagnostic_tools_example()
    await custom_policy_example()
    
    print("All examples completed!")


if __name__ == '__main__':
    asyncio.run(main())