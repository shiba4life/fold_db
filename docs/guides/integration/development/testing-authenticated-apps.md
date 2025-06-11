# Testing Authenticated Applications

Comprehensive guide to testing applications that use DataFold signature authentication. Covers unit testing, integration testing, mocking strategies, and automated test pipelines.

## 🎯 What You'll Learn

Advanced testing strategies for authenticated applications:
- 🧪 **Unit Testing** - Mock authentication for isolated component testing
- 🔗 **Integration Testing** - End-to-end testing with real authentication flows
- 🎭 **Mock Strategies** - Flexible authentication mocking for different scenarios
- ⚡ **Performance Testing** - Load testing authenticated endpoints
- 🏗️ **Test Infrastructure** - CI/CD pipeline integration and test environments
- 🔍 **Debugging Tests** - Troubleshooting authentication test failures

## ⏱️ Estimated Time: 45 minutes

## 🛠️ Prerequisites

- Completed framework integration tutorial ([React](../frameworks/frontend/react-integration-tutorial.md), [Node.js](../frameworks/backend/nodejs-express-tutorial.md), or [FastAPI](../frameworks/backend/python-fastapi-tutorial.md))
- Basic testing knowledge (Jest, pytest, or similar)
- Understanding of mocking and test doubles

## 🧪 JavaScript/TypeScript Testing

### Jest Setup for React/Node.js

```javascript
// jest.config.js
module.exports = {
  testEnvironment: 'node',
  setupFilesAfterEnv: ['<rootDir>/src/test/setup.js'],
  testMatch: ['**/__tests__/**/*.test.{js,ts}', '**/*.test.{js,ts}'],
  collectCoverageFrom: [
    'src/**/*.{js,ts}',
    '!src/**/*.d.ts',
    '!src/test/**',
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
};
```

### Authentication Mocking Utilities

```javascript
// src/test/mocks/datafold.js
import { jest } from '@jest/globals';

// Mock the entire DataFold SDK
export const mockDataFoldSDK = {
  generateKeyPair: jest.fn(),
  DataFoldClient: jest.fn(),
  createSigner: jest.fn(),
  verifySignature: jest.fn(),
};

// Mock key pair
export const mockKeyPair = {
  privateKey: 'mock-private-key-12345',
  publicKey: 'mock-public-key-67890',
};

// Mock authenticated client
export const mockAuthenticatedClient = {
  get: jest.fn(),
  post: jest.fn(),
  put: jest.fn(),
  delete: jest.fn(),
  testConnection: jest.fn(),
};

// Mock registration response
export const mockRegistrationResponse = {
  data: {
    client_id: 'mock-client-123',
    key_id: 'mock-key-456',
    status: 'active',
  },
};

// Mock authentication headers
export const mockAuthHeaders = {
  'signature': 'keyId="mock-key",algorithm="ed25519",headers="(request-target) content-digest",signature="mock-signature"',
  'signature-input': 'sig=("@method" "@target-uri" "content-digest");created=1625097600',
  'content-type': 'application/json',
};

// Authentication test helpers
export class AuthTestHelper {
  static mockSuccessfulAuth() {
    mockDataFoldSDK.generateKeyPair.mockResolvedValue(mockKeyPair);
    mockDataFoldSDK.DataFoldClient.mockImplementation(() => mockAuthenticatedClient);
    
    // Mock successful API responses
    mockAuthenticatedClient.get.mockResolvedValue({ data: [] });
    mockAuthenticatedClient.post.mockResolvedValue({ data: mockRegistrationResponse.data });
    mockAuthenticatedClient.testConnection.mockResolvedValue(true);
    
    return {
      client: mockAuthenticatedClient,
      keyPair: mockKeyPair,
      clientId: mockRegistrationResponse.data.client_id,
    };
  }

  static mockFailedAuth(errorMessage = 'Authentication failed') {
    mockDataFoldSDK.generateKeyPair.mockRejectedValue(new Error(errorMessage));
    mockDataFoldSDK.DataFoldClient.mockImplementation(() => {
      throw new Error(errorMessage);
    });
    
    return { error: errorMessage };
  }

  static mockNetworkError() {
    const networkError = new Error('Network request failed');
    networkError.code = 'NETWORK_ERROR';
    
    mockAuthenticatedClient.get.mockRejectedValue(networkError);
    mockAuthenticatedClient.post.mockRejectedValue(networkError);
    
    return { error: networkError };
  }

  static reset() {
    Object.values(mockDataFoldSDK).forEach(mock => {
      if (jest.isMockFunction(mock)) {
        mock.mockReset();
      }
    });
    
    Object.values(mockAuthenticatedClient).forEach(mock => {
      if (jest.isMockFunction(mock)) {
        mock.mockReset();
      }
    });
  }
}

// Mock signature verification middleware
export const mockSignatureMiddleware = (shouldAuthenticate = true) => {
  return (req, res, next) => {
    if (shouldAuthenticate) {
      req.datafold = {
        isAuthenticated: true,
        clientId: 'test-client-123',
        signature: {
          valid: true,
          timestamp: new Date(),
          components: ['method', 'target-uri', 'content-digest'],
        },
      };
    } else {
      req.datafold = {
        isAuthenticated: false,
        clientId: null,
      };
    }
    next();
  };
};
```

### Unit Testing Examples

```javascript
// src/test/unit/auth.test.js
import { AuthTestHelper } from '../mocks/datafold';
import { DataFoldService } from '../../services/datafold';

// Mock the DataFold SDK
jest.mock('@datafold/sdk', () => require('../mocks/datafold').mockDataFoldSDK);

describe('DataFold Authentication Service', () => {
  let service;

  beforeEach(() => {
    AuthTestHelper.reset();
    service = new DataFoldService();
  });

  describe('Initialization', () => {
    it('should initialize successfully with valid configuration', async () => {
      // Arrange
      const mockAuth = AuthTestHelper.mockSuccessfulAuth();

      // Act
      await service.initialize('https://api.datafold.com');

      // Assert
      expect(service.isAuthenticated()).toBe(true);
      expect(service.getClientId()).toBe(mockAuth.clientId);
    });

    it('should handle initialization failure gracefully', async () => {
      // Arrange
      AuthTestHelper.mockFailedAuth('Server unreachable');

      // Act & Assert
      await expect(service.initialize('https://invalid-url')).rejects.toThrow('Server unreachable');
      expect(service.isAuthenticated()).toBe(false);
    });

    it('should generate new credentials when none provided', async () => {
      // Arrange
      const mockAuth = AuthTestHelper.mockSuccessfulAuth();

      // Act
      await service.initialize('https://api.datafold.com');

      // Assert
      expect(mockDataFoldSDK.generateKeyPair).toHaveBeenCalled();
      expect(mockAuthenticatedClient.post).toHaveBeenCalledWith(
        expect.stringContaining('/api/crypto/keys/register'),
        expect.objectContaining({
          client_id: expect.any(String),
          public_key: mockAuth.keyPair.publicKey,
        })
      );
    });
  });

  describe('API Operations', () => {
    beforeEach(async () => {
      AuthTestHelper.mockSuccessfulAuth();
      await service.initialize('https://api.datafold.com');
    });

    it('should fetch schemas successfully', async () => {
      // Arrange
      const mockSchemas = [
        { name: 'schema1', fields: [] },
        { name: 'schema2', fields: [] },
      ];
      mockAuthenticatedClient.get.mockResolvedValue({ data: mockSchemas });

      // Act
      const schemas = await service.getSchemas();

      // Assert
      expect(schemas).toEqual(mockSchemas);
      expect(mockAuthenticatedClient.get).toHaveBeenCalledWith('/api/schemas');
    });

    it('should create schema successfully', async () => {
      // Arrange
      const newSchema = {
        name: 'test_schema',
        fields: [{ name: 'id', type: 'string', required: true }],
      };
      const createdSchema = { ...newSchema, id: 'schema-123' };
      mockAuthenticatedClient.post.mockResolvedValue({ data: createdSchema });

      // Act
      const result = await service.createSchema(newSchema.name, newSchema.fields);

      // Assert
      expect(result).toEqual(createdSchema);
      expect(mockAuthenticatedClient.post).toHaveBeenCalledWith('/api/schemas', {
        name: newSchema.name,
        fields: newSchema.fields,
        version: '1.0.0',
      });
    });

    it('should handle API errors gracefully', async () => {
      // Arrange
      const apiError = new Error('Schema already exists');
      apiError.status = 409;
      mockAuthenticatedClient.post.mockRejectedValue(apiError);

      // Act & Assert
      await expect(service.createSchema('duplicate', [])).rejects.toThrow('Schema already exists');
    });
  });

  describe('Connection Testing', () => {
    it('should report healthy connection', async () => {
      // Arrange
      AuthTestHelper.mockSuccessfulAuth();
      await service.initialize('https://api.datafold.com');
      mockAuthenticatedClient.testConnection.mockResolvedValue(true);

      // Act
      const isHealthy = await service.testConnection();

      // Assert
      expect(isHealthy).toBe(true);
    });

    it('should report unhealthy connection', async () => {
      // Arrange
      AuthTestHelper.mockSuccessfulAuth();
      await service.initialize('https://api.datafold.com');
      mockAuthenticatedClient.testConnection.mockRejectedValue(new Error('Connection failed'));

      // Act
      const isHealthy = await service.testConnection();

      // Assert
      expect(isHealthy).toBe(false);
    });
  });
});
```

### Integration Testing with Supertest

```javascript
// src/test/integration/api.test.js
import request from 'supertest';
import { createApp } from '../../app';
import { AuthTestHelper, mockSignatureMiddleware } from '../mocks/datafold';

describe('API Integration Tests', () => {
  let app;

  beforeAll(async () => {
    // Create app with test configuration
    process.env.NODE_ENV = 'test';
    process.env.DATAFOLD_ENABLE_AUTH = 'true';
    process.env.DATAFOLD_REQUIRE_AUTH = 'false';
    
    app = await createApp();
  });

  beforeEach(() => {
    AuthTestHelper.reset();
  });

  describe('Health Endpoints', () => {
    it('should return health status without authentication', async () => {
      const response = await request(app)
        .get('/health')
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.status).toBe('healthy');
    });

    it('should return detailed health with authentication info', async () => {
      const response = await request(app)
        .get('/api/health/detailed')
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data).toHaveProperty('authentication');
    });
  });

  describe('Authentication Flow', () => {
    it('should allow access to public endpoints without auth', async () => {
      const response = await request(app)
        .get('/api/auth/status')
        .expect(200);

      expect(response.body.data.isAuthenticated).toBe(false);
    });

    it('should require authentication for protected endpoints', async () => {
      await request(app)
        .get('/api/schemas')
        .expect(401);
    });

    it('should allow access with valid signature', async () => {
      // Mock successful authentication
      AuthTestHelper.mockSuccessfulAuth();

      const response = await request(app)
        .get('/api/schemas')
        .set('Signature', 'keyId="test",algorithm="ed25519",signature="valid"')
        .set('Signature-Input', 'sig=("@method" "@target-uri");created=1625097600')
        .expect(200);

      expect(response.body.success).toBe(true);
    });

    it('should reject requests with invalid signature', async () => {
      // Mock failed authentication
      AuthTestHelper.mockFailedAuth('Invalid signature');

      await request(app)
        .get('/api/schemas')
        .set('Signature', 'keyId="test",algorithm="ed25519",signature="invalid"')
        .set('Signature-Input', 'sig=("@method" "@target-uri");created=1625097600')
        .expect(401);
    });
  });

  describe('Schema Operations', () => {
    beforeEach(() => {
      // Mock successful authentication for all schema tests
      AuthTestHelper.mockSuccessfulAuth();
    });

    it('should list schemas', async () => {
      const mockSchemas = [{ name: 'test_schema', fields: [] }];
      mockAuthenticatedClient.get.mockResolvedValue({ data: mockSchemas });

      const response = await request(app)
        .get('/api/schemas')
        .set('Signature', 'keyId="test",algorithm="ed25519",signature="valid"')
        .set('Signature-Input', 'sig=("@method" "@target-uri");created=1625097600')
        .expect(200);

      expect(response.body.data).toEqual(mockSchemas);
    });

    it('should create new schema', async () => {
      const newSchema = {
        name: 'user_events',
        fields: [
          { name: 'user_id', type: 'string', required: true },
          { name: 'event_type', type: 'string', required: true },
        ],
      };

      mockAuthenticatedClient.post.mockResolvedValue({ 
        data: { ...newSchema, id: 'schema-123' } 
      });

      const response = await request(app)
        .post('/api/schemas')
        .set('Signature', 'keyId="test",algorithm="ed25519",signature="valid"')
        .set('Signature-Input', 'sig=("@method" "@target-uri" "content-digest");created=1625097600')
        .send(newSchema)
        .expect(201);

      expect(response.body.success).toBe(true);
      expect(response.body.data.name).toBe(newSchema.name);
    });

    it('should validate schema data', async () => {
      const invalidSchema = {
        name: '', // Invalid: empty name
        fields: [],
      };

      await request(app)
        .post('/api/schemas')
        .set('Signature', 'keyId="test",algorithm="ed25519",signature="valid"')
        .set('Signature-Input', 'sig=("@method" "@target-uri" "content-digest");created=1625097600')
        .send(invalidSchema)
        .expect(400);
    });
  });

  describe('Error Handling', () => {
    it('should handle DataFold service errors', async () => {
      AuthTestHelper.mockNetworkError();

      const response = await request(app)
        .get('/api/schemas')
        .set('Signature', 'keyId="test",algorithm="ed25519",signature="valid"')
        .set('Signature-Input', 'sig=("@method" "@target-uri");created=1625097600')
        .expect(500);

      expect(response.body.success).toBe(false);
      expect(response.body.error.message).toContain('Network request failed');
    });

    it('should return 404 for unknown routes', async () => {
      const response = await request(app)
        .get('/api/nonexistent')
        .expect(404);

      expect(response.body.success).toBe(false);
      expect(response.body.error.code).toBe('not_found');
    });
  });
});
```

## 🐍 Python Testing with pytest

### Pytest Configuration

```python
# pytest.ini
[tool:pytest]
testpaths = tests
python_files = test_*.py
python_classes = Test*
python_functions = test_*
addopts = 
    --strict-markers
    --cov=app
    --cov-report=term-missing
    --cov-report=html
    --cov-fail-under=80
markers =
    unit: Unit tests
    integration: Integration tests
    slow: Slow tests
    auth: Authentication tests
```

### Authentication Test Fixtures

```python
# tests/conftest.py
import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from fastapi.testclient import TestClient
from httpx import AsyncClient

from app.main import create_app
from app.services.datafold import DataFoldService
from app.models.auth import AuthenticatedUser, SignatureInfo
from datetime import datetime

@pytest.fixture
def mock_datafold_sdk():
    """Mock the entire DataFold SDK"""
    with patch('app.services.datafold.generate_key_pair') as mock_gen_keys, \
         patch('app.services.datafold.create_enhanced_http_client') as mock_client:
        
        # Mock key generation
        mock_gen_keys.return_value = (
            b'mock_private_key_bytes',
            b'mock_public_key_bytes'
        )
        
        # Mock HTTP client
        mock_http_client = AsyncMock()
        mock_http_client.get.return_value = {'data': []}
        mock_http_client.post.return_value = {'data': {'client_id': 'test-client-123'}}
        mock_http_client.delete.return_value = None
        mock_client.return_value = mock_http_client
        
        yield {
            'generate_key_pair': mock_gen_keys,
            'create_client': mock_client,
            'http_client': mock_http_client
        }

@pytest.fixture
def authenticated_user():
    """Create a mock authenticated user"""
    return AuthenticatedUser(
        client_id="test-client-123",
        signature=SignatureInfo(
            valid=True,
            algorithm="ed25519",
            components=["method", "target-uri", "content-digest"],
            timestamp=datetime.now(),
            key_id="test-key-456"
        ),
        metadata={"test": True}
    )

@pytest.fixture
def mock_auth_dependency(authenticated_user):
    """Mock authentication dependency"""
    async def mock_get_current_user():
        return authenticated_user
    
    async def mock_get_current_user_optional():
        return authenticated_user
    
    return {
        'require_auth': mock_get_current_user,
        'optional_auth': mock_get_current_user_optional
    }

@pytest.fixture
def test_app(mock_datafold_sdk):
    """Create test FastAPI app"""
    app = create_app()
    return app

@pytest.fixture
def client(test_app):
    """Create test client"""
    return TestClient(test_app)

@pytest.fixture
async def async_client(test_app):
    """Create async test client"""
    async with AsyncClient(app=test_app, base_url="http://test") as ac:
        yield ac

@pytest.fixture
def mock_signature_headers():
    """Mock signature headers for requests"""
    return {
        'Signature': 'keyId="test-key",algorithm="ed25519",headers="(request-target) content-digest",signature="mock-signature"',
        'Signature-Input': 'sig=("@method" "@target-uri" "content-digest");created=1625097600',
        'Content-Type': 'application/json'
    }

class AuthTestHelper:
    """Helper class for authentication testing"""
    
    @staticmethod
    def mock_successful_verification():
        """Mock successful signature verification"""
        with patch('app.services.datafold.DataFoldService.verify_signature') as mock:
            mock.return_value = AuthenticatedUser(
                client_id="test-client-123",
                signature=SignatureInfo(
                    valid=True,
                    algorithm="ed25519",
                    components=["method", "target-uri"],
                    timestamp=datetime.now()
                )
            )
            return mock
    
    @staticmethod
    def mock_failed_verification():
        """Mock failed signature verification"""
        with patch('app.services.datafold.DataFoldService.verify_signature') as mock:
            mock.return_value = None
            return mock

@pytest.fixture
def auth_helper():
    """Authentication test helper"""
    return AuthTestHelper
```

### Unit Testing Examples

```python
# tests/unit/test_auth_service.py
import pytest
from unittest.mock import AsyncMock, patch
from app.services.datafold import DataFoldService
from app.config import Settings

class TestDataFoldService:
    """Test DataFold service functionality"""
    
    @pytest.mark.asyncio
    async def test_initialization_success(self, mock_datafold_sdk):
        """Test successful service initialization"""
        # Arrange
        service = DataFoldService()
        
        # Act
        await service.initialize()
        
        # Assert
        assert service.is_ready()
        assert service.client_id == 'test-client-123'
        mock_datafold_sdk['generate_key_pair'].assert_called_once()
    
    @pytest.mark.asyncio
    async def test_initialization_with_existing_credentials(self, mock_datafold_sdk):
        """Test initialization with existing credentials"""
        # Arrange
        service = DataFoldService()
        service.settings.datafold_client_id = 'existing-client'
        service.settings.datafold_private_key = 'abcd1234'
        
        # Act
        await service.initialize()
        
        # Assert
        assert service.is_ready()
        mock_datafold_sdk['generate_key_pair'].assert_not_called()
    
    @pytest.mark.asyncio
    async def test_initialization_failure(self, mock_datafold_sdk):
        """Test initialization failure handling"""
        # Arrange
        service = DataFoldService()
        mock_datafold_sdk['generate_key_pair'].side_effect = Exception("Connection failed")
        
        # Act & Assert
        with pytest.raises(Exception, match="Connection failed"):
            await service.initialize()
        
        assert not service.is_ready()
    
    @pytest.mark.asyncio
    async def test_get_schemas(self, mock_datafold_sdk):
        """Test schema retrieval"""
        # Arrange
        service = DataFoldService()
        await service.initialize()
        
        mock_schemas = [
            {'name': 'schema1', 'fields': []},
            {'name': 'schema2', 'fields': []}
        ]
        mock_datafold_sdk['http_client'].get.return_value = {'data': mock_schemas}
        
        # Act
        schemas = await service.get_schemas()
        
        # Assert
        assert schemas == mock_schemas
        mock_datafold_sdk['http_client'].get.assert_called_with('/api/schemas')
    
    @pytest.mark.asyncio
    async def test_create_schema(self, mock_datafold_sdk):
        """Test schema creation"""
        # Arrange
        service = DataFoldService()
        await service.initialize()
        
        schema_data = {
            'name': 'test_schema',
            'fields': [{'name': 'id', 'type': 'string', 'required': True}]
        }
        created_schema = {**schema_data, 'id': 'schema-123'}
        mock_datafold_sdk['http_client'].post.return_value = {'data': created_schema}
        
        # Act
        result = await service.create_schema(schema_data)
        
        # Assert
        assert result == created_schema
        mock_datafold_sdk['http_client'].post.assert_called_with('/api/schemas', json=schema_data)
    
    @pytest.mark.asyncio
    async def test_signature_verification_success(self, mock_datafold_sdk):
        """Test successful signature verification"""
        # Arrange
        service = DataFoldService()
        await service.initialize()
        
        request_data = {
            'method': 'GET',
            'url': 'https://api.example.com/test',
            'headers': {'signature': 'valid-signature'},
            'signature': 'valid-signature',
            'signature_input': 'sig=("@method" "@target-uri");created=1625097600'
        }
        
        # Act
        user = await service.verify_signature(request_data)
        
        # Assert
        assert user is not None
        assert user.client_id == 'test-client-123'
        assert user.signature.valid is True
    
    @pytest.mark.asyncio
    async def test_health_status(self, mock_datafold_sdk):
        """Test health status reporting"""
        # Arrange
        service = DataFoldService()
        await service.initialize()
        
        # Act
        health = await service.get_health_status()
        
        # Assert
        assert health['service'] == 'datafold'
        assert health['status'] == 'healthy'
        assert health['client_id'] == 'test-client-123'

# tests/unit/test_auth_dependencies.py
import pytest
from fastapi import HTTPException
from unittest.mock import AsyncMock
from app.auth.dependencies import SignatureAuth
from app.services.datafold import DataFoldService

class TestAuthDependencies:
    """Test authentication dependencies"""
    
    @pytest.mark.asyncio
    async def test_signature_auth_success(self, authenticated_user, mock_signature_headers):
        """Test successful signature authentication"""
        # Arrange
        mock_request = AsyncMock()
        mock_request.method = 'GET'
        mock_request.url = 'https://api.example.com/test'
        mock_request.headers = mock_signature_headers
        
        mock_datafold_service = AsyncMock()
        mock_datafold_service.verify_signature.return_value = authenticated_user
        
        mock_settings = AsyncMock()
        mock_settings.datafold_enable_auth = True
        
        auth = SignatureAuth(required=True)
        
        # Act
        result = await auth(
            request=mock_request,
            credentials=None,
            datafold_service=mock_datafold_service,
            settings=mock_settings
        )
        
        # Assert
        assert result == authenticated_user
        mock_datafold_service.verify_signature.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_signature_auth_missing_headers(self):
        """Test authentication with missing signature headers"""
        # Arrange
        mock_request = AsyncMock()
        mock_request.headers = {}  # No signature headers
        
        mock_datafold_service = AsyncMock()
        mock_settings = AsyncMock()
        mock_settings.datafold_enable_auth = True
        
        auth = SignatureAuth(required=True)
        
        # Act & Assert
        with pytest.raises(HTTPException) as exc_info:
            await auth(
                request=mock_request,
                credentials=None,
                datafold_service=mock_datafold_service,
                settings=mock_settings
            )
        
        assert exc_info.value.status_code == 401
        assert "Missing signature headers" in str(exc_info.value.detail)
    
    @pytest.mark.asyncio
    async def test_signature_auth_disabled(self):
        """Test authentication when disabled"""
        # Arrange
        mock_request = AsyncMock()
        mock_datafold_service = AsyncMock()
        mock_settings = AsyncMock()
        mock_settings.datafold_enable_auth = False
        
        auth = SignatureAuth(required=True)
        
        # Act
        result = await auth(
            request=mock_request,
            credentials=None,
            datafold_service=mock_datafold_service,
            settings=mock_settings
        )
        
        # Assert
        assert result is None
        mock_datafold_service.verify_signature.assert_not_called()
```

### Integration Testing

```python
# tests/integration/test_api_endpoints.py
import pytest
from fastapi.testclient import TestClient
from unittest.mock import patch

class TestAPIEndpoints:
    """Test API endpoint integration"""
    
    def test_health_endpoint(self, client):
        """Test health endpoint"""
        response = client.get("/health/")
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert "timestamp" in data["data"]
    
    def test_auth_status_unauthenticated(self, client):
        """Test auth status without authentication"""
        response = client.get("/api/auth/status")
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert data["data"]["is_authenticated"] is False
    
    def test_protected_endpoint_without_auth(self, client):
        """Test protected endpoint without authentication"""
        response = client.get("/api/auth/me")
        
        assert response.status_code == 401
        data = response.json()
        assert data["success"] is False
    
    @patch('app.services.datafold.DataFoldService.verify_signature')
    def test_protected_endpoint_with_auth(self, mock_verify, client, authenticated_user, mock_signature_headers):
        """Test protected endpoint with authentication"""
        # Arrange
        mock_verify.return_value = authenticated_user
        
        # Act
        response = client.get("/api/auth/me", headers=mock_signature_headers)
        
        # Assert
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert data["data"]["client_id"] == authenticated_user.client_id
    
    @patch('app.services.datafold.DataFoldService.get_schemas')
    @patch('app.services.datafold.DataFoldService.verify_signature')
    def test_list_schemas(self, mock_verify, mock_get_schemas, client, authenticated_user, mock_signature_headers):
        """Test schema listing"""
        # Arrange
        mock_verify.return_value = authenticated_user
        mock_schemas = [{"name": "test_schema", "fields": []}]
        mock_get_schemas.return_value = mock_schemas
        
        # Act
        response = client.get("/api/schemas/", headers=mock_signature_headers)
        
        # Assert
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert data["data"] == mock_schemas
    
    @patch('app.services.datafold.DataFoldService.create_schema')
    @patch('app.services.datafold.DataFoldService.verify_signature')
    def test_create_schema(self, mock_verify, mock_create, client, authenticated_user, mock_signature_headers):
        """Test schema creation"""
        # Arrange
        mock_verify.return_value = authenticated_user
        new_schema = {
            "name": "user_events",
            "fields": [
                {"name": "user_id", "type": "string", "required": True},
                {"name": "event_type", "type": "string", "required": True}
            ]
        }
        created_schema = {**new_schema, "id": "schema-123"}
        mock_create.return_value = created_schema
        
        # Act
        response = client.post("/api/schemas/", json=new_schema, headers=mock_signature_headers)
        
        # Assert
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert data["data"]["name"] == new_schema["name"]
    
    def test_schema_validation(self, client, mock_signature_headers):
        """Test schema data validation"""
        invalid_schema = {
            "name": "",  # Invalid: empty name
            "fields": []
        }
        
        response = client.post("/api/schemas/", json=invalid_schema, headers=mock_signature_headers)
        
        assert response.status_code == 422  # Validation error
```

## 🧪 Performance Testing

### Load Testing with Locust

```python
# tests/performance/locustfile.py
from locust import HttpUser, task, between
import json
import random

class AuthenticatedAPIUser(HttpUser):
    wait_time = between(1, 3)
    
    def on_start(self):
        """Setup for each user"""
        self.headers = {
            'Signature': 'keyId="load-test",algorithm="ed25519",headers="(request-target) content-digest",signature="test-signature"',
            'Signature-Input': 'sig=("@method" "@target-uri" "content-digest");created=1625097600',
            'Content-Type': 'application/json'
        }
    
    @task(10)
    def health_check(self):
        """Test health endpoint (high frequency)"""
        self.client.get("/health/")
    
    @task(5)
    def auth_status(self):
        """Test auth status endpoint"""
        self.client.get("/api/auth/status")
    
    @task(3)
    def list_schemas(self):
        """Test schema listing"""
        self.client.get("/api/schemas/", headers=self.headers)
    
    @task(1)
    def create_schema(self):
        """Test schema creation (low frequency)"""
        schema_name = f"load_test_schema_{random.randint(1000, 9999)}"
        schema_data = {
            "name": schema_name,
            "fields": [
                {"name": "id", "type": "string", "required": True},
                {"name": "timestamp", "type": "datetime", "required": True}
            ]
        }
        
        self.client.post("/api/schemas/", 
                        json=schema_data, 
                        headers=self.headers)

class UnauthenticatedUser(HttpUser):
    wait_time = between(2, 5)
    
    @task
    def public_endpoints(self):
        """Test public endpoints"""
        endpoints = ["/health/", "/api/auth/status"]
        endpoint = random.choice(endpoints)
        self.client.get(endpoint)
```

### Signature Performance Testing

```javascript
// tests/performance/signature-bench.js
import { performance } from 'perf_hooks';
import { generateKeyPair, createSigner } from '@datafold/sdk';

class SignatureBenchmark {
  constructor() {
    this.results = [];
  }

  async setup() {
    this.keyPair = await generateKeyPair();
    this.signer = createSigner({
      algorithm: 'ed25519',
      keyId: 'benchmark-key',
      privateKey: this.keyPair.privateKey
    });
  }

  async benchmarkSigning(iterations = 1000) {
    console.log(`Running signature benchmark with ${iterations} iterations...`);
    
    const requests = [];
    for (let i = 0; i < iterations; i++) {
      requests.push({
        method: 'POST',
        url: `/api/test/${i}`,
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ test: i, timestamp: Date.now() })
      });
    }

    const start = performance.now();
    
    for (const request of requests) {
      await this.signer.sign(request);
    }

    const end = performance.now();
    const totalTime = end - start;
    const avgTime = totalTime / iterations;

    this.results.push({
      operation: 'signing',
      iterations,
      totalTime,
      avgTime,
      signaturesPerSecond: 1000 / avgTime
    });

    console.log(`Signing benchmark results:`);
    console.log(`  Total time: ${totalTime.toFixed(2)}ms`);
    console.log(`  Average time per signature: ${avgTime.toFixed(2)}ms`);
    console.log(`  Signatures per second: ${(1000 / avgTime).toFixed(2)}`);
  }

  async benchmarkVerification(iterations = 1000) {
    console.log(`Running verification benchmark with ${iterations} iterations...`);
    
    // Pre-generate signed requests
    const signedRequests = [];
    for (let i = 0; i < iterations; i++) {
      const request = {
        method: 'GET',
        url: `/api/verify/${i}`,
        headers: { 'content-type': 'application/json' }
      };
      
      const signedRequest = await this.signer.sign(request);
      signedRequests.push(signedRequest);
    }

    const start = performance.now();
    
    for (const signedRequest of signedRequests) {
      // In a real scenario, this would be server-side verification
      await this.verifySignature(signedRequest);
    }

    const end = performance.now();
    const totalTime = end - start;
    const avgTime = totalTime / iterations;

    this.results.push({
      operation: 'verification',
      iterations,
      totalTime,
      avgTime,
      verificationsPerSecond: 1000 / avgTime
    });

    console.log(`Verification benchmark results:`);
    console.log(`  Total time: ${totalTime.toFixed(2)}ms`);
    console.log(`  Average time per verification: ${avgTime.toFixed(2)}ms`);
    console.log(`  Verifications per second: ${(1000 / avgTime).toFixed(2)}`);
  }

  async verifySignature(signedRequest) {
    // Mock verification - in practice this would use the actual verification logic
    return new Promise(resolve => {
      setTimeout(resolve, Math.random() * 2); // Simulate verification time
    });
  }

  generateReport() {
    console.log('\n=== Performance Benchmark Report ===');
    this.results.forEach(result => {
      console.log(`${result.operation.toUpperCase()}:`);
      console.log(`  Operations per second: ${result.signaturesPerSecond || result.verificationsPerSecond}`);
      console.log(`  Average latency: ${result.avgTime.toFixed(2)}ms`);
    });
  }
}

// Run benchmarks
async function runBenchmarks() {
  const benchmark = new SignatureBenchmark();
  await benchmark.setup();
  
  await benchmark.benchmarkSigning(1000);
  await benchmark.benchmarkVerification(1000);
  
  benchmark.generateReport();
}

runBenchmarks().catch(console.error);
```

## 🏗️ CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        node-version: [16.x, 18.x, 20.x]
        python-version: [3.8, 3.9, '3.10', '3.11']

    steps:
    - uses: actions/checkout@v3

    # Node.js setup
    - name: Use Node.js ${{ matrix.node-version }}
      uses: actions/setup-node@v3
      with:
        node-version: ${{ matrix.node-version }}
        cache: 'npm'

    # Python setup
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v4
      with:
        python-version: ${{ matrix.python-version }}

    # Install dependencies
    - name: Install Node.js dependencies
      run: npm ci

    - name: Install Python dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -r requirements.txt
        pip install pytest-cov pytest-asyncio

    # Environment setup
    - name: Set up test environment
      env:
        DATAFOLD_SERVER_URL: https://test-api.datafold.com
        DATAFOLD_ENABLE_AUTH: true
        DATAFOLD_REQUIRE_AUTH: false
        NODE_ENV: test
      run: |
        echo "Test environment configured"

    # Run tests
    - name: Run JavaScript/TypeScript tests
      run: |
        npm run test:unit
        npm run test:integration
      env:
        CI: true
        DATAFOLD_SERVER_URL: https://test-api.datafold.com

    - name: Run Python tests
      run: |
        pytest tests/unit --cov=app --cov-report=xml
        pytest tests/integration --cov=app --cov-append --cov-report=xml

    # Performance tests (optional)
    - name: Run performance tests
      if: github.event_name == 'push' && github.ref == 'refs/heads/main'
      run: |
        npm run test:performance
        python tests/performance/signature_benchmark.py

    # Upload coverage
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage.xml
        flags: unittests
        name: codecov-umbrella

  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3

    - name: Run security audit
      run: |
        npm audit --audit-level moderate
        pip install safety
        safety check

    - name: Run SAST scan
      uses: github/super-linter@v4
      env:
        DEFAULT_BRANCH: main
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Test Environment Configuration

```yaml
# docker-compose.test.yml
version: '3.8'

services:
  test-api:
    build:
      context: .
      dockerfile: Dockerfile.test
    environment:
      - NODE_ENV=test
      - DATAFOLD_ENABLE_AUTH=true
      - DATAFOLD_REQUIRE_AUTH=false
      - DATAFOLD_SERVER_URL=http://mock-datafold:8080
    depends_on:
      - mock-datafold
    volumes:
      - ./tests:/app/tests
      - ./coverage:/app/coverage

  mock-datafold:
    image: wiremock/wiremock:latest
    ports:
      - "8080:8080"
    volumes:
      - ./tests/mocks/wiremock:/home/wiremock
    command: ["--global-response-templating", "--verbose"]

  test-runner:
    build:
      context: .
      dockerfile: Dockerfile.test
    command: ["npm", "run", "test:all"]
    environment:
      - CI=true
      - DATAFOLD_SERVER_URL=http://mock-datafold:8080
    depends_on:
      - test-api
      - mock-datafold
    volumes:
      - ./tests:/app/tests
      - ./coverage:/app/coverage
```

## 🔧 Debugging Test Failures

### Debug Configuration

```javascript
// debug-tests.js
import debug from 'debug';

// Enable debugging for specific modules
debug.enabled = (namespace) => {
  return namespace.startsWith('datafold:') || 
         namespace.startsWith('test:') ||
         process.env.DEBUG_ALL === 'true';
};

export const debugAuth = debug('datafold:auth');
export const debugTest = debug('test:runner');
export const debugAPI = debug('datafold:api');

// Usage in tests
debugTest('Starting authentication test suite');
debugAuth('Mocking signature verification: %o', mockData);
```

### Test Failure Analysis

```javascript
// tests/utils/test-diagnostics.js
export class TestDiagnostics {
  static async captureAuthState(service) {
    return {
      isInitialized: service.isInitialized,
      isAuthenticated: service.isAuthenticated(),
      clientId: service.getClientId(),
      lastError: service.getLastError(),
      connectionStatus: await service.testConnection().catch(e => e.message)
    };
  }

  static async analyzeRequestFailure(request, response, expectedAuth = true) {
    const analysis = {
      request: {
        method: request.method,
        url: request.url,
        hasSignature: !!request.headers.signature,
        hasSignatureInput: !!request.headers['signature-input']
      },
      response: {
        status: response.status,
        hasAuthError: response.status === 401,
        errorMessage: response.data?.error?.message
      },
      expectedAuth,
      possibleIssues: []
    };

    // Analyze potential issues
    if (expectedAuth && !analysis.request.hasSignature) {
      analysis.possibleIssues.push('Missing signature header');
    }

    if (expectedAuth && !analysis.request.hasSignatureInput) {
      analysis.possibleIssues.push('Missing signature-input header');
    }

    if (analysis.response.hasAuthError && expectedAuth) {
      analysis.possibleIssues.push('Authentication failed - check signature validity');
    }

    return analysis;
  }

  static generateTestReport(testResults) {
    const report = {
      summary: {
        total: testResults.length,
        passed: testResults.filter(t => t.passed).length,
        failed: testResults.filter(t => !t.passed).length
      },
      failures: testResults.filter(t => !t.passed).map(t => ({
        test: t.name,
        error: t.error,
        analysis: t.analysis
      }))
    };

    console.log('Test Report:', JSON.stringify(report, null, 2));
    return report;
  }
}
```

## 🎯 Best Practices

### 1. Test Organization
```
tests/
├── unit/           # Fast, isolated tests
├── integration/    # API and database tests  
├── e2e/           # End-to-end user scenarios
├── performance/   # Load and stress tests
├── fixtures/      # Test data and mocks
└── utils/         # Test utilities and helpers
```

### 2. Mock Strategy Guidelines
- **Unit tests**: Mock all external dependencies
- **Integration tests**: Mock only external services
- **E2E tests**: Use real services or high-fidelity mocks

### 3. Authentication Test Patterns
- Test both successful and failed authentication
- Verify proper error handling and status codes
- Test edge cases (expired signatures, malformed headers)
- Validate security headers and CORS policies

### 4. Performance Test Guidelines
- Establish baseline performance metrics
- Test under realistic load conditions
- Monitor authentication latency impact
- Validate signature caching effectiveness

## 🔗 Next Steps

- **[Debugging Authentication Issues](debugging-signature-auth.md)** - Troubleshooting guide
- **[CI/CD Integration](../deployment/ci-cd-integration-tutorial.md)** - Automated testing pipelines
- **[Production Monitoring](../deployment/monitoring-integration.md)** - Live system monitoring

---

🎉 **Congratulations!** You now have comprehensive testing strategies for DataFold signature authentication. Your applications are thoroughly tested and ready for production deployment with confidence.

💡 **Pro Tips**:
- Always test authentication edge cases
- Use realistic test data and scenarios
- Monitor test performance and flakiness
- Implement proper test isolation
- Validate security properties in tests
- Keep authentication mocks up-to-date with real implementation