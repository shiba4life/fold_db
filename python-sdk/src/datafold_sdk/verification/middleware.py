"""
Verification middleware for HTTP responses and requests

This module provides middleware for automatic signature verification in various
Python web frameworks and HTTP clients.
"""

import asyncio
import time
from typing import Dict, List, Optional, Union, Any, Callable, Awaitable, Pattern
from dataclasses import dataclass
import re

from .types import (
    VerificationConfig,
    VerificationResult,
    VerifiableResponse,
    VerificationError,
    VerificationPolicy,
    VerificationStatus,
)
from .verifier import RFC9421Verifier, create_verifier
from .policies import VERIFICATION_POLICIES
from ..signing.types import SignableRequest, HttpMethod


@dataclass
class ResponseVerificationConfig:
    """Response verification middleware configuration"""
    verification_config: VerificationConfig
    default_policy: Optional[str] = None
    throw_on_failure: bool = False
    on_verification_failure: Optional[Callable[[VerificationResult, Any], None]] = None
    skip_patterns: Optional[List[Pattern]] = None
    enable_perf_monitoring: bool = True


@dataclass
class RequestVerificationConfig:
    """Request verification middleware configuration"""
    verification_config: VerificationConfig
    default_policy: Optional[str] = None
    reject_invalid: bool = False
    on_validation_result: Optional[Callable[[VerificationResult, Any], None]] = None


class ResponseVerificationMiddleware:
    """Middleware for verifying HTTP response signatures"""
    
    def __init__(self, config: ResponseVerificationConfig):
        self.config = config
        self.verifier = create_verifier(config.verification_config)
    
    async def __call__(self, response: Any) -> Any:
        """
        Verify response signature
        
        Args:
            response: HTTP response object
            
        Returns:
            Response object (potentially modified)
        """
        # Check if we should skip verification
        if self.config.skip_patterns and hasattr(response, 'url'):
            url = getattr(response, 'url', '')
            for pattern in self.config.skip_patterns:
                if pattern.search(url):
                    return response
        
        try:
            # Extract headers
            headers = self._extract_headers(response)
            
            # Check if response has signature headers
            if 'signature-input' not in headers or 'signature' not in headers:
                # No signature to verify
                return response
            
            # Get response body
            body = await self._get_response_body(response)
            
            # Create verifiable response
            verifiable_response = VerifiableResponse(
                status=getattr(response, 'status', getattr(response, 'status_code', 200)),
                headers=headers,
                body=body,
                url=getattr(response, 'url', ''),
                method=getattr(response, 'method', 'GET')
            )
            
            # Perform verification
            result = await self.verifier.verify(
                verifiable_response,
                headers,
                policy=self.config.default_policy
            )
            
            # Handle verification result
            if result.status != VerificationStatus.VALID or not result.signature_valid:
                if self.config.on_verification_failure:
                    self.config.on_verification_failure(result, response)
                
                if self.config.throw_on_failure:
                    raise VerificationError(
                        f'Response signature verification failed: {result.error["message"] if result.error else "Unknown error"}',
                        'RESPONSE_VERIFICATION_FAILED',
                        {
                            'url': getattr(response, 'url', ''),
                            'status': getattr(response, 'status', 0),
                            'verification_result': result.__dict__
                        }
                    )
            
            # Add verification result as metadata if possible
            if hasattr(response, '__dict__'):
                response._verification_result = result
            
            return response
            
        except Exception as error:
            if isinstance(error, VerificationError):
                raise error
            
            if self.config.throw_on_failure:
                raise VerificationError(
                    f'Response verification middleware failed: {error}',
                    'MIDDLEWARE_ERROR',
                    {
                        'original_error': str(error),
                        'url': getattr(response, 'url', '')
                    }
                )
            
            # Log error but don't fail the request
            import logging
            logger = logging.getLogger(__name__)
            logger.warning(f'Response verification middleware error: {error}')
            return response
    
    def _extract_headers(self, response: Any) -> Dict[str, str]:
        """Extract headers from response object"""
        headers = {}
        
        if hasattr(response, 'headers'):
            response_headers = response.headers
            if hasattr(response_headers, 'items'):
                # requests.Response style
                for key, value in response_headers.items():
                    headers[key.lower()] = value
            elif hasattr(response_headers, '__iter__'):
                # aiohttp style
                for key, value in response_headers:
                    headers[key.lower()] = value
        
        return headers
    
    async def _get_response_body(self, response: Any) -> Optional[str]:
        """Get response body as string"""
        try:
            if hasattr(response, 'text'):
                # requests.Response or aiohttp.ClientResponse
                if asyncio.iscoroutinefunction(response.text):
                    return await response.text()
                else:
                    return response.text
            elif hasattr(response, 'content'):
                # requests.Response style
                content = response.content
                if isinstance(content, bytes):
                    return content.decode('utf-8')
                return str(content)
            elif hasattr(response, 'body'):
                body = response.body
                if isinstance(body, bytes):
                    return body.decode('utf-8')
                return str(body)
        except Exception:
            pass
        
        return None


class RequestVerificationMiddleware:
    """Middleware for verifying HTTP request signatures"""
    
    def __init__(self, config: RequestVerificationConfig):
        self.config = config
        self.verifier = create_verifier(config.verification_config)
    
    async def __call__(self, request: Any) -> Dict[str, Any]:
        """
        Verify request signature
        
        Args:
            request: HTTP request object
            
        Returns:
            dict: Verification result with validation info
        """
        try:
            # Extract headers
            headers = self._extract_headers(request)
            
            # Check if request has signature headers
            if 'signature-input' not in headers or 'signature' not in headers:
                return self._create_no_signature_result()
            
            # Get request body
            body = await self._get_request_body(request)
            
            # Create signable request
            method_str = str(getattr(request, 'method', 'GET'))
            try:
                method = HttpMethod(method_str.upper())
            except ValueError:
                method = HttpMethod.GET
            
            signable_request = SignableRequest(
                method=method,
                url=self._get_request_url(request),
                headers=headers,
                body=body
            )
            
            # Perform verification
            result = await self.verifier.verify(
                signable_request,
                headers,
                policy=self.config.default_policy
            )
            
            # Handle verification result
            if self.config.on_validation_result:
                self.config.on_validation_result(result, request)
            
            valid = result.status == VerificationStatus.VALID and result.signature_valid
            should_reject = not valid and self.config.reject_invalid
            
            return {
                'valid': valid,
                'result': result,
                'should_reject': should_reject
            }
            
        except Exception as error:
            error_result = VerificationResult.create_error(
                'MIDDLEWARE_ERROR',
                str(error)
            )
            
            return {
                'valid': False,
                'result': error_result,
                'should_reject': self.config.reject_invalid
            }
    
    def _extract_headers(self, request: Any) -> Dict[str, str]:
        """Extract headers from request object"""
        headers = {}
        
        if hasattr(request, 'headers'):
            request_headers = request.headers
            if hasattr(request_headers, 'items'):
                for key, value in request_headers.items():
                    headers[key.lower()] = value
            elif hasattr(request_headers, '__iter__'):
                for key, value in request_headers:
                    headers[key.lower()] = value
        elif hasattr(request, 'META'):
            # Django request style
            for key, value in request.META.items():
                if key.startswith('HTTP_'):
                    header_name = key[5:].replace('_', '-').lower()
                    headers[header_name] = value
        
        return headers
    
    async def _get_request_body(self, request: Any) -> Optional[str]:
        """Get request body as string"""
        try:
            if hasattr(request, 'body'):
                body = request.body
                if isinstance(body, bytes):
                    return body.decode('utf-8')
                return str(body) if body is not None else None
            elif hasattr(request, 'data'):
                # Some frameworks use 'data'
                data = request.data
                if isinstance(data, bytes):
                    return data.decode('utf-8')
                return str(data) if data is not None else None
        except Exception:
            pass
        
        return None
    
    def _get_request_url(self, request: Any) -> str:
        """Get request URL"""
        if hasattr(request, 'url'):
            return request.url
        elif hasattr(request, 'build_absolute_uri'):
            # Django request
            return request.build_absolute_uri()
        elif hasattr(request, 'path'):
            # Basic path
            return request.path
        
        return ''
    
    def _create_no_signature_result(self) -> Dict[str, Any]:
        """Create result for requests without signatures"""
        result = VerificationResult(
            status=VerificationStatus.UNKNOWN,
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
            diagnostics=VerificationResult.create_error('NO_SIGNATURE', 'No signature present').diagnostics,
            performance={'total_time': 0, 'step_timings': {}}
        )
        
        return {
            'valid': False,
            'result': result,
            'should_reject': False
        }


# Framework-specific middleware creators

def create_response_verification_middleware(config: ResponseVerificationConfig) -> ResponseVerificationMiddleware:
    """
    Create response verification middleware
    
    Args:
        config: Response verification configuration
        
    Returns:
        ResponseVerificationMiddleware: Middleware instance
    """
    return ResponseVerificationMiddleware(config)


def create_request_verification_middleware(config: RequestVerificationConfig) -> RequestVerificationMiddleware:
    """
    Create request verification middleware
    
    Args:
        config: Request verification configuration
        
    Returns:
        RequestVerificationMiddleware: Middleware instance
    """
    return RequestVerificationMiddleware(config)


def create_django_verification_middleware(config: RequestVerificationConfig):
    """
    Create Django verification middleware
    
    Args:
        config: Request verification configuration
        
    Returns:
        Django middleware class
    """
    middleware = create_request_verification_middleware(config)
    
    class DjangoVerificationMiddleware:
        def __init__(self, get_response):
            self.get_response = get_response
            self.middleware = middleware
        
        def __call__(self, request):
            # Sync wrapper for async verification
            import asyncio
            
            try:
                loop = asyncio.get_event_loop()
            except RuntimeError:
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
            
            verification_result = loop.run_until_complete(self.middleware(request))
            
            # Add verification result to request
            request.signature_verification = verification_result
            
            if verification_result['should_reject']:
                try:
                    from django.http import JsonResponse  # type: ignore
                    return JsonResponse({
                        'error': 'Signature verification failed',
                        'code': 'INVALID_SIGNATURE',
                        'details': verification_result['result'].error
                    }, status=401)
                except ImportError:
                    # Django not available, return simple dict
                    return {
                        'error': 'Signature verification failed',
                        'code': 'INVALID_SIGNATURE',
                        'details': verification_result['result'].error
                    }
            
            response = self.get_response(request)
            return response
    
    return DjangoVerificationMiddleware


def create_flask_verification_middleware(config: RequestVerificationConfig):
    """
    Create Flask verification middleware
    
    Args:
        config: Request verification configuration
        
    Returns:
        Flask before_request function
    """
    middleware = create_request_verification_middleware(config)
    
    def flask_verification_middleware():
        import asyncio
        try:
            from flask import request, jsonify  # type: ignore
            import flask  # type: ignore
        except ImportError:
            raise ImportError("Flask is required for Flask middleware")
        
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
        
        verification_result = loop.run_until_complete(middleware(request))
        
        # Store verification result in flask.g or request context
        flask.g.signature_verification = verification_result
        
        if verification_result['should_reject']:
            return jsonify({
                'error': 'Signature verification failed',
                'code': 'INVALID_SIGNATURE',
                'details': verification_result['result'].error
            }), 401
    
    return flask_verification_middleware


def create_fastapi_verification_middleware(config: RequestVerificationConfig):
    """
    Create FastAPI verification middleware
    
    Args:
        config: Request verification configuration
        
    Returns:
        FastAPI middleware function
    """
    middleware = create_request_verification_middleware(config)
    
    async def fastapi_verification_middleware(request, call_next):
        verification_result = await middleware(request)
        
        # Add verification result to request state
        request.state.signature_verification = verification_result
        
        if verification_result['should_reject']:
            try:
                from fastapi import HTTPException  # type: ignore
                raise HTTPException(
                    status_code=401,
                    detail={
                        'error': 'Signature verification failed',
                        'code': 'INVALID_SIGNATURE',
                        'details': verification_result['result'].error
                    }
                )
            except ImportError:
                raise Exception("FastAPI is required for FastAPI middleware")
        
        response = await call_next(request)
        return response
    
    return fastapi_verification_middleware


# Batch verification utilities

class BatchVerifier:
    """Batch verification utility for multiple requests/responses"""
    
    def __init__(self, config: VerificationConfig):
        self.verifier = create_verifier(config)
    
    async def verify_batch(
        self,
        items: List[Dict[str, Any]]
    ) -> List[VerificationResult]:
        """
        Verify multiple requests/responses
        
        Args:
            items: List of verification items
            
        Returns:
            List[VerificationResult]: Verification results
        """
        verification_tasks = []
        
        for item in items:
            task = self.verifier.verify(
                item['message'],
                item['headers'],
                item.get('policy'),
                item.get('public_key'),
                item.get('key_id'),
                item.get('skip_key_retrieval', False)
            )
            verification_tasks.append(task)
        
        results = await asyncio.gather(*verification_tasks, return_exceptions=True)
        
        processed_results = []
        for result in results:
            if isinstance(result, VerificationResult):
                processed_results.append(result)
            elif isinstance(result, Exception):
                error_result = VerificationResult.create_error(
                    'BATCH_VERIFICATION_ERROR',
                    f'Batch verification failed: {result}'
                )
                processed_results.append(error_result)
            else:
                error_result = VerificationResult.create_error(
                    'BATCH_VERIFICATION_ERROR',
                    'Unknown batch verification error'
                )
                processed_results.append(error_result)
        
        return processed_results
    
    def get_batch_stats(self, results: List[VerificationResult]) -> Dict[str, Any]:
        """
        Get batch verification statistics
        
        Args:
            results: Verification results
            
        Returns:
            dict: Batch statistics
        """
        total = len(results)
        valid = len([r for r in results if r.status == VerificationStatus.VALID])
        invalid = len([r for r in results if r.status == VerificationStatus.INVALID])
        errors = len([r for r in results if r.status == VerificationStatus.ERROR])
        
        total_time = sum(r.performance.get('total_time', 0) for r in results)
        average_time = total_time / total if total > 0 else 0
        
        return {
            'total': total,
            'valid': valid,
            'invalid': invalid,
            'errors': errors,
            'success_rate': valid / total if total > 0 else 0,
            'average_time': average_time,
            'total_time': total_time
        }


def create_batch_verifier(config: VerificationConfig) -> BatchVerifier:
    """
    Create a batch verifier instance
    
    Args:
        config: Verification configuration
        
    Returns:
        BatchVerifier: Batch verifier instance
    """
    return BatchVerifier(config)


# Policy-based middleware

def create_policy_based_middleware(
    policies: Dict[str, str],  # URL pattern -> policy name mapping
    config: ResponseVerificationConfig
) -> Callable:
    """
    Create policy-based verification middleware
    
    Args:
        policies: URL pattern to policy name mapping
        config: Response verification configuration
        
    Returns:
        Middleware function
    """
    async def policy_middleware(response: Any) -> Any:
        # Find matching policy
        policy_name = 'standard'  # default
        url = getattr(response, 'url', '')
        
        for pattern, policy in policies.items():
            if re.search(pattern, url):
                policy_name = policy
                break
        
        # Create middleware with specific policy
        middleware_config = ResponseVerificationConfig(
            verification_config=config.verification_config,
            default_policy=policy_name,
            throw_on_failure=config.throw_on_failure,
            on_verification_failure=config.on_verification_failure,
            skip_patterns=config.skip_patterns,
            enable_perf_monitoring=config.enable_perf_monitoring
        )
        
        middleware = create_response_verification_middleware(middleware_config)
        return await middleware(response)
    
    return policy_middleware


# Utility functions

def create_skip_patterns(patterns: List[str]) -> List[Pattern]:
    """
    Create compiled regex patterns for skipping verification
    
    Args:
        patterns: List of regex pattern strings
        
    Returns:
        List[Pattern]: Compiled regex patterns
    """
    return [re.compile(pattern) for pattern in patterns]


async def verify_response_with_middleware(
    response: Any,
    config: ResponseVerificationConfig
) -> Any:
    """
    Verify single response using middleware
    
    Args:
        response: Response to verify
        config: Verification configuration
        
    Returns:
        Verified response
    """
    middleware = create_response_verification_middleware(config)
    return await middleware(response)


async def verify_request_with_middleware(
    request: Any,
    config: RequestVerificationConfig
) -> Dict[str, Any]:
    """
    Verify single request using middleware
    
    Args:
        request: Request to verify
        config: Verification configuration
        
    Returns:
        Verification result
    """
    middleware = create_request_verification_middleware(config)
    return await middleware(request)