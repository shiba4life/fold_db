# DataFold Python SDK

The DataFold Python SDK provides a complete implementation of RFC 9421 HTTP Message Signatures for Python applications. It offers a simple, type-safe API for authenticating requests to DataFold services with full async/await support.

## 🚀 Quick Start

### Installation

```bash
# pip
pip install datafold-sdk

# Poetry
poetry add datafold-sdk

# pipenv
pipenv install datafold-sdk

# conda
conda install -c conda-forge datafold-sdk
```

### Basic Usage

```python
from datafold_sdk import DataFoldClient, generate_keypair

# Generate Ed25519 keypair
private_key, public_key = generate_keypair()

# Create authenticated client
client = DataFoldClient(
    server_url='https://api.datafold.com',
    client_id='your-client-id',
    private_key=private_key
)

# Make authenticated requests
schemas = client.get('/api/schemas')
new_schema = client.post('/api/schemas', json=schema_data)
```

### Async Usage

```python
import asyncio
from datafold_sdk import AsyncDataFoldClient, generate_keypair

async def main():
    # Generate keypair
    private_key, public_key = generate_keypair()
    
    # Create async client
    async with AsyncDataFoldClient(
        server_url='https://api.datafold.com',
        client_id='your-client-id',
        private_key=private_key
    ) as client:
        # Make authenticated requests
        schemas = await client.get('/api/schemas')
        new_schema = await client.post('/api/schemas', json=schema_data)

asyncio.run(main())
```

## 📦 Package Contents

The SDK provides several modules for different use cases:

### Core Modules
- **[`DataFoldClient`](api-reference.md#datafoldclient)** - Synchronous HTTP client with authentication
- **[`AsyncDataFoldClient`](api-reference.md#asyncdatafoldclient)** - Asynchronous HTTP client with authentication  
- **[`RFC9421Signer`](api-reference.md#rfc9421signer)** - Low-level request signing
- **[`generate_keypair`](api-reference.md#generate-keypair)** - Ed25519 key generation

### Crypto Modules
- **[`Ed25519`](api-reference.md#ed25519)** - Ed25519 cryptographic operations
- **[`KeyDerivation`](api-reference.md#keyderivation)** - Key derivation utilities
- **[`SecureStorage`](api-reference.md#securestorage)** - Secure key storage

### Utility Modules
- **[`Validation`](api-reference.md#validation)** - Input validation helpers
- **[`Utils`](api-reference.md#utils)** - Common utilities
- **[`Types`](api-reference.md#types)** - Type definitions

## 🎯 Platform Support

### Python Versions
- **Python 3.8+**: Full support with type hints
- **Python 3.9+**: Enhanced performance optimizations
- **Python 3.10+**: Pattern matching support
- **Python 3.11+**: Improved asyncio performance
- **Python 3.12+**: Latest features and optimizations

### Operating Systems
- **Linux**: ✅ Full support (all distributions)
- **macOS**: ✅ Intel and Apple Silicon
- **Windows**: ✅ Windows 10+ with WSL support
- **Docker**: ✅ Official Docker images available

### Framework Compatibility
- **Django**: ✅ Middleware and integration utilities
- **Flask**: ✅ Extension and decorators
- **FastAPI**: ✅ Dependency injection and middleware
- **Starlette**: ✅ ASGI middleware
- **aiohttp**: ✅ Native async support
- **Tornado**: ✅ RequestHandler integration
- **Celery**: ✅ Task integration
- **pytest**: ✅ Testing utilities

## 🔧 Configuration

### Basic Configuration

```python
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    # Required
    server_url='https://api.datafold.com',
    client_id='your-client-id',
    private_key=private_key_bytes,
    
    # Optional
    timeout=10.0,           # Request timeout (seconds)
    retries=3,              # Retry attempts
    security_profile='standard',  # Security profile
    debug=False             # Debug logging
)
```

### Advanced Configuration

```python
from datafold_sdk import DataFoldClient, SecurityProfiles, SigningConfig

client = DataFoldClient(
    server_url='https://api.datafold.com',
    client_id='your-client-id',
    private_key=private_key_bytes,
    
    # Security settings
    security_profile=SecurityProfiles.STRICT,
    
    # Signing configuration
    signing_config=SigningConfig(
        algorithm='ed25519',
        components=SignatureComponents(
            method=True,
            target_uri=True,
            headers=['content-type', 'content-digest'],
            content_digest=True
        ),
        timestamp_generator=lambda: int(time.time()),
        nonce_generator=lambda: str(uuid.uuid4()).replace('-', '')
    ),
    
    # HTTP configuration
    http_config=HTTPConfig(
        timeout=15.0,
        retries=5,
        retry_delay=1.0,
        user_agent='MyApp/1.0.0 DataFoldSDK/2.0.0',
        verify_ssl=True,
        cert=None,  # Client certificate
        proxies=None
    ),
    
    # Interceptors
    request_interceptor=lambda req: print(f'Request: {req.method} {req.url}'),
    response_interceptor=lambda resp: print(f'Response: {resp.status_code}')
)
```

### Environment-Specific Configuration

```python
import os
from datafold_sdk import DataFoldClient

# Development
dev_client = DataFoldClient(
    server_url='http://localhost:9001',
    client_id='dev-client',
    private_key=bytes.fromhex(os.environ['DEV_PRIVATE_KEY']),
    security_profile='lenient',
    debug=True
)

# Production
prod_client = DataFoldClient(
    server_url='https://api.datafold.com',
    client_id='prod-client',
    private_key=bytes.fromhex(os.environ['PROD_PRIVATE_KEY']),
    security_profile='strict',
    debug=False,
    timeout=5.0
)
```

## 🔐 Authentication Setup

### 1. Generate Keys

```python
from datafold_sdk.crypto import generate_keypair

# Generate new Ed25519 keypair
private_key, public_key = generate_keypair()

print(f'Private Key (keep secret!): {private_key.hex()}')
print(f'Public Key (register with server): {public_key.hex()}')

# Save keys securely
import os
os.environ['DATAFOLD_PRIVATE_KEY'] = private_key.hex()
os.environ['DATAFOLD_PUBLIC_KEY'] = public_key.hex()
```

### 2. Register Public Key

```python
from datafold_sdk import register_public_key

registration = register_public_key(
    server_url='https://api.datafold.com',
    client_id='my-app-client',
    public_key=public_key,
    key_name='Production Key',
    metadata={
        'environment': 'production',
        'version': '1.0.0',
        'service': 'data-pipeline'
    }
)

print('Registration successful:', registration)
```

### 3. Create Authenticated Client

```python
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    server_url='https://api.datafold.com',
    client_id=registration['client_id'],
    private_key=private_key
)
```

## 💻 Usage Examples

### Synchronous API Operations

```python
from datafold_sdk import DataFoldClient

# Create client
client = DataFoldClient(config)

# Get all schemas
schemas = client.get('/api/schemas')
print(f'Found {len(schemas.json())} schemas')

# Get specific schema
schema = client.get('/api/schemas/user_events')
print(f'Schema: {schema.json()["name"]}')

# Create new schema
new_schema = client.post('/api/schemas', json={
    'name': 'product_events',
    'version': '1.0.0',
    'fields': [
        {'name': 'product_id', 'type': 'string', 'required': True},
        {'name': 'event_type', 'type': 'string', 'required': True},
        {'name': 'timestamp', 'type': 'datetime', 'required': True}
    ]
})

print(f'Created schema: {new_schema.json()["id"]}')

# Update schema
updated_schema = client.put('/api/schemas/product_events', json={
    'version': '1.1.0',
    'fields': [
        {'name': 'product_id', 'type': 'string', 'required': True},
        {'name': 'event_type', 'type': 'string', 'required': True},
        {'name': 'timestamp', 'type': 'datetime', 'required': True},
        {'name': 'user_id', 'type': 'string', 'required': False}
    ]
})

# Delete schema
client.delete('/api/schemas/old_schema')
print('Schema deleted successfully')
```

### Asynchronous API Operations

```python
import asyncio
from datafold_sdk import AsyncDataFoldClient

async def main():
    async with AsyncDataFoldClient(config) as client:
        # Concurrent requests
        tasks = [
            client.get('/api/schemas'),
            client.get('/api/schemas/user_events'),
            client.get('/api/schemas/product_events')
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                print(f'Request {i} failed: {result}')
            else:
                print(f'Request {i} succeeded: {result.status_code}')

asyncio.run(main())
```

### Data Validation

```python
# Validate single record
validation = client.post('/api/schemas/user_events/validate', json={
    'data': {
        'user_id': 'user123',
        'event_type': 'page_view',
        'timestamp': '2025-06-09T23:27:09Z',
        'page_url': 'https://example.com/products'
    },
    'options': {
        'strict': True,
        'return_errors': True
    }
})

result = validation.json()
if result['valid']:
    print('Data is valid!')
else:
    print('Validation errors:', result['errors'])

# Validate batch of records
batch_validation = client.post('/api/schemas/user_events/validate/batch', json={
    'records': [
        {'user_id': 'user1', 'event_type': 'login', 'timestamp': '2025-06-09T10:00:00Z'},
        {'user_id': 'user2', 'event_type': 'logout', 'timestamp': '2025-06-09T11:00:00Z'},
        {'user_id': 'user3', 'event_type': 'purchase', 'timestamp': '2025-06-09T12:00:00Z'}
    ],
    'options': {
        'fail_fast': False,
        'return_details': True
    }
})

batch_result = batch_validation.json()
print(f'Validated {batch_result["total_records"]} records')
print(f'{batch_result["valid_records"]} valid, {batch_result["invalid_records"]} invalid')
```

## 🌐 Framework Integration

### Django Integration

```python
# settings.py
DATAFOLD_CONFIG = {
    'SERVER_URL': 'https://api.datafold.com',
    'CLIENT_ID': 'django-app',
    'PRIVATE_KEY': os.environ['DATAFOLD_PRIVATE_KEY'],
    'SECURITY_PROFILE': 'standard'
}

# datafold_client.py
from django.conf import settings
from datafold_sdk import DataFoldClient

class DataFoldService:
    def __init__(self):
        self.client = DataFoldClient(
            server_url=settings.DATAFOLD_CONFIG['SERVER_URL'],
            client_id=settings.DATAFOLD_CONFIG['CLIENT_ID'],
            private_key=bytes.fromhex(settings.DATAFOLD_CONFIG['PRIVATE_KEY']),
            security_profile=settings.DATAFOLD_CONFIG['SECURITY_PROFILE']
        )
    
    def validate_model_data(self, model_name, data):
        """Validate Django model data against DataFold schema"""
        response = self.client.post(f'/api/schemas/{model_name}/validate', json={
            'data': data,
            'options': {'strict': True, 'return_errors': True}
        })
        return response.json()

# Global instance
datafold_service = DataFoldService()

# models.py
from django.db import models
from django.core.exceptions import ValidationError
from .datafold_client import datafold_service

class UserEvent(models.Model):
    user_id = models.CharField(max_length=100)
    event_type = models.CharField(max_length=50)
    timestamp = models.DateTimeField()
    data = models.JSONField()
    
    def clean(self):
        """Validate against DataFold schema before saving"""
        model_data = {
            'user_id': self.user_id,
            'event_type': self.event_type,
            'timestamp': self.timestamp.isoformat(),
            'data': self.data
        }
        
        validation = datafold_service.validate_model_data('user_events', model_data)
        
        if not validation['valid']:
            raise ValidationError(f'DataFold validation failed: {validation["errors"]}')

# views.py
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from django.views.decorators.http import require_http_methods
import json
from .datafold_client import datafold_service

@csrf_exempt
@require_http_methods(['POST'])
def validate_data(request):
    try:
        data = json.loads(request.body)
        schema_name = data.get('schema')
        record_data = data.get('data')
        
        validation = datafold_service.validate_model_data(schema_name, record_data)
        
        return JsonResponse({
            'valid': validation['valid'],
            'errors': validation.get('errors', [])
        })
        
    except Exception as e:
        return JsonResponse({'error': str(e)}, status=500)
```

### Flask Integration

```python
# app.py
from flask import Flask, request, jsonify
from datafold_sdk import DataFoldClient
import os

app = Flask(__name__)

# Initialize DataFold client
client = DataFoldClient(
    server_url=os.environ['DATAFOLD_SERVER_URL'],
    client_id=os.environ['DATAFOLD_CLIENT_ID'],
    private_key=bytes.fromhex(os.environ['DATAFOLD_PRIVATE_KEY'])
)

@app.route('/api/validate', methods=['POST'])
def validate_data():
    try:
        data = request.get_json()
        schema_name = data.get('schema')
        record_data = data.get('data')
        
        response = client.post(f'/api/schemas/{schema_name}/validate', json={
            'data': record_data,
            'options': {'strict': True, 'return_errors': True}
        })
        
        validation = response.json()
        
        return jsonify({
            'valid': validation['valid'],
            'errors': validation.get('errors', [])
        })
        
    except Exception as e:
        app.logger.error(f'Validation error: {e}')
        return jsonify({'error': 'Validation failed'}), 500

@app.route('/api/schemas', methods=['GET'])
def list_schemas():
    try:
        response = client.get('/api/schemas')
        return jsonify(response.json())
        
    except Exception as e:
        app.logger.error(f'Schema listing error: {e}')
        return jsonify({'error': 'Failed to list schemas'}), 500

if __name__ == '__main__':
    app.run(debug=True)
```

### FastAPI Integration

```python
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel
from typing import Dict, List, Any, Optional
from datafold_sdk import AsyncDataFoldClient
import os

app = FastAPI(title="DataFold Integration API")

# Pydantic models
class ValidationRequest(BaseModel):
    schema: str
    data: Dict[str, Any]
    options: Optional[Dict[str, Any]] = {}

class ValidationResponse(BaseModel):
    valid: bool
    errors: List[str] = []
    warnings: List[str] = []

# Dependency to get DataFold client
async def get_datafold_client():
    async with AsyncDataFoldClient(
        server_url=os.environ['DATAFOLD_SERVER_URL'],
        client_id=os.environ['DATAFOLD_CLIENT_ID'],
        private_key=bytes.fromhex(os.environ['DATAFOLD_PRIVATE_KEY'])
    ) as client:
        yield client

@app.post("/api/validate", response_model=ValidationResponse)
async def validate_data(
    request: ValidationRequest,
    client: AsyncDataFoldClient = Depends(get_datafold_client)
):
    """Validate data against a DataFold schema"""
    try:
        response = await client.post(f'/api/schemas/{request.schema}/validate', json={
            'data': request.data,
            'options': {
                'strict': True,
                'return_errors': True,
                'return_warnings': True,
                **request.options
            }
        })
        
        validation = response.json()
        
        return ValidationResponse(
            valid=validation['valid'],
            errors=validation.get('errors', []),
            warnings=validation.get('warnings', [])
        )
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Validation failed: {e}")

@app.get("/api/schemas")
async def list_schemas(client: AsyncDataFoldClient = Depends(get_datafold_client)):
    """List all available schemas"""
    try:
        response = await client.get('/api/schemas')
        return response.json()
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to list schemas: {e}")

# Middleware for request logging
@app.middleware("http")
async def log_requests(request, call_next):
    import time
    start_time = time.time()
    
    response = await call_next(request)
    
    process_time = time.time() - start_time
    print(f"{request.method} {request.url} - {response.status_code} - {process_time:.3f}s")
    
    return response

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Celery Integration

```python
# tasks.py
from celery import Celery
from datafold_sdk import DataFoldClient
import os

app = Celery('datafold_tasks')

# Configure DataFold client
client = DataFoldClient(
    server_url=os.environ['DATAFOLD_SERVER_URL'],
    client_id=os.environ['DATAFOLD_CLIENT_ID'],
    private_key=bytes.fromhex(os.environ['DATAFOLD_PRIVATE_KEY'])
)

@app.task(bind=True, autoretry_for=(Exception,), retry_kwargs={'max_retries': 3})
def validate_batch_data(self, schema_name, records):
    """Validate a batch of records asynchronously"""
    try:
        response = client.post(f'/api/schemas/{schema_name}/validate/batch', json={
            'records': records,
            'options': {
                'fail_fast': False,
                'return_details': True
            }
        })
        
        result = response.json()
        
        return {
            'task_id': self.request.id,
            'total_records': result['total_records'],
            'valid_records': result['valid_records'],
            'invalid_records': result['invalid_records'],
            'details': result.get('details', [])
        }
        
    except Exception as e:
        self.retry(countdown=60, exc=e)

@app.task
def upload_schema(schema_data):
    """Upload a new schema to DataFold"""
    try:
        response = client.post('/api/schemas', json=schema_data)
        return {
            'status': 'success',
            'schema_id': response.json()['id'],
            'message': f'Schema {schema_data["name"]} uploaded successfully'
        }
        
    except Exception as e:
        return {
            'status': 'error',
            'message': f'Schema upload failed: {e}'
        }

# Usage
if __name__ == '__main__':
    # Start validation task
    task = validate_batch_data.delay('user_events', [
        {'user_id': 'user1', 'event_type': 'login'},
        {'user_id': 'user2', 'event_type': 'logout'}
    ])
    
    print(f'Task started: {task.id}')
    
    # Get result
    result = task.get(timeout=30)
    print(f'Validation complete: {result}')
```

## 🔧 Advanced Usage

### Custom HTTP Session

```python
import requests
from datafold_sdk.signing import RFC9421Signer

# Create custom session with connection pooling
session = requests.Session()
adapter = requests.adapters.HTTPAdapter(
    pool_connections=20,
    pool_maxsize=50,
    max_retries=3
)
session.mount('https://', adapter)
session.mount('http://', adapter)

# Create signer
signer = RFC9421Signer(
    algorithm='ed25519',
    key_id='my-client-id',
    private_key=private_key_bytes,
    components=SignatureComponents(
        method=True,
        target_uri=True,
        headers=['content-type', 'content-digest'],
        content_digest=True
    )
)

def authenticated_request(method, url, **kwargs):
    """Make authenticated request with custom session"""
    # Prepare request
    req = requests.Request(method, url, **kwargs)
    prepared = session.prepare_request(req)
    
    # Convert to signable format
    signable_request = SignableRequest(
        method=prepared.method,
        url=prepared.url,
        headers=dict(prepared.headers),
        body=prepared.body
    )
    
    # Sign request
    signature_result = signer.sign_request(signable_request)
    
    # Apply signature headers
    prepared.headers.update(signature_result.headers)
    
    # Send request
    return session.send(prepared)

# Use custom authenticated request
response = authenticated_request('POST', 'https://api.datafold.com/api/schemas', json=schema_data)
```

### Context Manager for Resource Management

```python
from datafold_sdk import DataFoldClient
from contextlib import contextmanager

@contextmanager
def datafold_transaction(client, rollback_on_error=True):
    """Context manager for DataFold operations with rollback support"""
    operations = []
    
    try:
        # Monkey patch client methods to track operations
        original_post = client.post
        original_put = client.put
        original_delete = client.delete
        
        def tracked_post(url, **kwargs):
            result = original_post(url, **kwargs)
            operations.append(('POST', url, result))
            return result
            
        def tracked_put(url, **kwargs):
            result = original_put(url, **kwargs)
            operations.append(('PUT', url, result))
            return result
            
        def tracked_delete(url, **kwargs):
            result = original_delete(url, **kwargs)
            operations.append(('DELETE', url, result))
            return result
        
        client.post = tracked_post
        client.put = tracked_put
        client.delete = tracked_delete
        
        yield client
        
    except Exception as e:
        if rollback_on_error:
            # Attempt to rollback operations in reverse order
            for method, url, result in reversed(operations):
                try:
                    if method == 'POST':
                        # Try to delete created resource
                        resource_id = result.json().get('id')
                        if resource_id:
                            original_delete(f"{url}/{resource_id}")
                    elif method == 'PUT':
                        # PUT operations are harder to rollback
                        pass
                    elif method == 'DELETE':
                        # DELETE operations cannot be easily rolled back
                        pass
                except Exception as rollback_error:
                    print(f"Rollback failed for {method} {url}: {rollback_error}")
        
        raise e
        
    finally:
        # Restore original methods
        client.post = original_post
        client.put = original_put
        client.delete = original_delete

# Usage
client = DataFoldClient(config)

with datafold_transaction(client) as tx_client:
    # All operations within this block are tracked
    schema = tx_client.post('/api/schemas', json=schema_data)
    validation = tx_client.post('/api/schemas/test/validate', json=test_data)
    # If any operation fails, previous operations are rolled back
```

### Async Context Manager with Connection Pooling

```python
import asyncio
import aiohttp
from datafold_sdk import AsyncDataFoldClient

class PooledAsyncDataFoldClient:
    def __init__(self, config, pool_size=20):
        self.config = config
        self.pool_size = pool_size
        self._session = None
        self._client = None
    
    async def __aenter__(self):
        # Create connection pool
        connector = aiohttp.TCPConnector(
            limit=self.pool_size,
            limit_per_host=10,
            keepalive_timeout=30,
            enable_cleanup_closed=True
        )
        
        timeout = aiohttp.ClientTimeout(total=self.config.get('timeout', 30))
        
        self._session = aiohttp.ClientSession(
            connector=connector,
            timeout=timeout
        )
        
        # Create DataFold client with custom session
        self._client = AsyncDataFoldClient(
            **self.config,
            session=self._session
        )
        
        await self._client.__aenter__()
        return self._client
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._client:
            await self._client.__aexit__(exc_type, exc_val, exc_tb)
        
        if self._session:
            await self._session.close()

# Usage
async def main():
    config = {
        'server_url': 'https://api.datafold.com',
        'client_id': 'async-client',
        'private_key': private_key_bytes
    }
    
    async with PooledAsyncDataFoldClient(config, pool_size=50) as client:
        # Make many concurrent requests efficiently
        tasks = [
            client.get('/api/schemas'),
            client.get('/api/schemas/user_events'),
            client.get('/api/schemas/product_events'),
            # ... many more requests
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        for result in results:
            if isinstance(result, Exception):
                print(f'Request failed: {result}')
            else:
                print(f'Request succeeded: {result.status}')

asyncio.run(main())
```

## 📊 Performance Optimization

### Connection Pooling

```python
from datafold_sdk import DataFoldClient
import requests.adapters

client = DataFoldClient(
    server_url='https://api.datafold.com',
    client_id='your-client-id',
    private_key=private_key_bytes,
    
    session_config={
        'pool_connections': 20,
        'pool_maxsize': 50,
        'pool_block': False,
        'retries': 3
    }
)
```

### Batch Processing

```python
from datafold_sdk.utils import BatchProcessor
import asyncio

async def process_records(records):
    """Process records in batches for optimal performance"""
    async with AsyncDataFoldClient(config) as client:
        processor = BatchProcessor(
            client=client,
            batch_size=100,
            max_concurrency=10,
            retry_attempts=3
        )
        
        # Process in batches
        async for batch_result in processor.process_batches(records, '/api/schemas/events/validate'):
            print(f'Processed batch: {batch_result.success_count}/{batch_result.total_count}')
            
            if batch_result.errors:
                print(f'Errors: {batch_result.errors}')

# Usage
records = [{'user_id': f'user{i}', 'event': 'test'} for i in range(1000)]
asyncio.run(process_records(records))
```

### Caching

```python
from datafold_sdk.cache import CachedDataFoldClient
import redis

# Redis cache backend
redis_client = redis.Redis(host='localhost', port=6379, db=0)

client = CachedDataFoldClient(
    server_url='https://api.datafold.com',
    client_id='your-client-id',
    private_key=private_key_bytes,
    
    cache_config={
        'backend': 'redis',
        'client': redis_client,
        'ttl': 300,  # 5 minutes
        'key_prefix': 'datafold_cache:',
        'cache_get_requests': True,
        'cache_post_requests': False
    }
)

# GET requests are cached automatically
schemas = client.get('/api/schemas')  # Hits server
schemas_cached = client.get('/api/schemas')  # Returns cached result
```

## 🧪 Testing

### Unit Testing with pytest

```python
import pytest
from unittest.mock import Mock, patch
from datafold_sdk import DataFoldClient, generate_keypair
from datafold_sdk.signing import RFC9421Signer

@pytest.fixture
def keypair():
    """Generate test keypair"""
    return generate_keypair()

@pytest.fixture
def client(keypair):
    """Create test client"""
    private_key, public_key = keypair
    return DataFoldClient(
        server_url='http://localhost:9001',
        client_id='test-client',
        private_key=private_key
    )

def test_keypair_generation():
    """Test Ed25519 keypair generation"""
    private_key, public_key = generate_keypair()
    
    assert isinstance(private_key, bytes)
    assert len(private_key) == 32
    assert isinstance(public_key, bytes)
    assert len(public_key) == 32

def test_request_signing(keypair):
    """Test RFC 9421 request signing"""
    private_key, public_key = keypair
    
    signer = RFC9421Signer(
        algorithm='ed25519',
        key_id='test-client',
        private_key=private_key
    )
    
    request = SignableRequest(
        method='POST',
        url='http://localhost:9001/api/test',
        headers={'content-type': 'application/json'},
        body='{"test": true}'
    )
    
    result = signer.sign_request(request)
    
    assert 'signature-input' in result.headers
    assert 'signature' in result.headers
    assert result.headers['signature'].startswith('sig1=:')
    assert result.headers['signature'].endswith(':')

@patch('requests.Session.request')
def test_authenticated_request(mock_request, client):
    """Test authenticated API request"""
    # Mock successful response
    mock_response = Mock()
    mock_response.status_code = 200
    mock_response.json.return_value = {'schemas': []}
    mock_request.return_value = mock_response
    
    # Make request
    response = client.get('/api/schemas')
    
    # Verify request was signed
    args, kwargs = mock_request.call_args
    headers = kwargs['headers']
    
    assert 'signature-input' in headers
    assert 'signature' in headers
    assert response.status_code == 200

@pytest.mark.asyncio
async def test_async_client(keypair):
    """Test async client functionality"""
    private_key, public_key = keypair
    
    with patch('aiohttp.ClientSession.request') as mock_request:
        # Mock async response
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={'schemas': []})
        mock_request.return_value.__aenter__.return_value = mock_response
        
        async with AsyncDataFoldClient(
            server_url='http://localhost:9001',
            client_id='test-client',
            private_key=private_key
        ) as client:
            response = await client.get('/api/schemas')
            assert response.status == 200
```

### Integration Testing

```python
import pytest
import asyncio
from datafold_sdk import DataFoldClient, AsyncDataFoldClient, generate_keypair, register_public_key

@pytest.fixture(scope="session")
def event_loop():
    """Create event loop for session"""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

@pytest.fixture(scope="session")
async def test_client():
    """Create test client with registered keys"""
    # Generate test keypair
    private_key, public_key = generate_keypair()
    
    # Register public key
    registration = register_public_key(
        server_url='http://localhost:9001',
        client_id='integration-test',
        public_key=public_key,
        key_name='Integration Test Key'
    )
    
    # Create client
    client = DataFoldClient(
        server_url='http://localhost:9001',
        client_id=registration['client_id'],
        private_key=private_key
    )
    
    yield client

def test_schema_operations(test_client):
    """Test full schema CRUD operations"""
    # List schemas
    schemas = test_client.get('/api/schemas')
    assert schemas.status_code == 200
    
    # Create schema
    schema_data = {
        'name': f'test_schema_{int(time.time())}',
        'version': '1.0.0',
        'fields': [
            {'name': 'id', 'type': 'string', 'required': True},
            {'name': 'name', 'type': 'string', 'required': True}
        ]
    }
    
    created_schema = test_client.post('/api/schemas', json=schema_data)
    assert created_schema.status_code == 201
    
    schema_name = created_schema.json()['name']
    
    # Get schema
    retrieved_schema = test_client.get(f'/api/schemas/{schema_name}')
    assert retrieved_schema.status_code == 200
    assert retrieved_schema.json()['name'] == schema_name
    
    # Update schema
    updated_data = {**schema_data, 'version': '1.1.0'}
    updated_schema = test_client.put(f'/api/schemas/{schema_name}', json=updated_data)
    assert updated_schema.status_code == 200
    
    # Delete schema
    delete_response = test_client.delete(f'/api/schemas/{schema_name}')
    assert delete_response.status_code == 204

@pytest.mark.asyncio
async def test_concurrent_requests():
    """Test concurrent request handling"""
    async with AsyncDataFoldClient(
        server_url='http://localhost:9001',
        client_id='integration-test',
        private_key=private_key
    ) as client:
        # Make multiple concurrent requests
        tasks = [
            client.get('/api/schemas'),
            client.get('/api/schemas'),
            client.get('/api/schemas')
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # All requests should succeed
        for result in results:
            assert not isinstance(result, Exception)
            assert result.status == 200
```

## 📝 Type Hints and IDE Support

The DataFold Python SDK includes comprehensive type hints for better IDE support:

```python
from datafold_sdk import DataFoldClient
from datafold_sdk.types import SchemaData, ValidationResult
from typing import List, Dict, Any

def process_schemas(client: DataFoldClient) -> List[SchemaData]:
    """Process schemas with full type safety"""
    response = client.get('/api/schemas')
    schemas: List[SchemaData] = response.json()
    
    return [schema for schema in schemas if schema.version.startswith('1.')]

def validate_records(
    client: DataFoldClient, 
    schema_name: str, 
    records: List[Dict[str, Any]]
) -> ValidationResult:
    """Validate records with type checking"""
    response = client.post(f'/api/schemas/{schema_name}/validate/batch', json={
        'records': records,
        'options': {'strict': True, 'return_errors': True}
    })
    
    return ValidationResult.from_dict(response.json())
```

## 🔗 Related Documentation

- **[API Reference](api-reference.md)** - Complete API documentation
- **[Examples](examples.md)** - Working code examples  
- **[Integration Guide](integration-guide.md)** - Framework integration
- **[JavaScript SDK](../javascript/README.md)** - JavaScript implementation
- **[CLI Tool](../cli/README.md)** - Command-line interface

## 📞 Support

- **Documentation**: [API Reference](api-reference.md)
- **Examples**: [Usage Examples](examples.md)
- **Issues**: [GitHub Issues](https://github.com/datafold/python-sdk/issues)
- **PyPI**: [Package Page](https://pypi.org/project/datafold-sdk/)

---

**Next**: Explore the [complete API reference](api-reference.md) or check out [practical examples](examples.md).