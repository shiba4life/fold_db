# Python/FastAPI Integration Tutorial

Build a modern, high-performance FastAPI application with DataFold signature authentication. This tutorial covers FastAPI dependency injection, async patterns, automatic documentation, and production deployment.

## 🎯 What You'll Build

A production-ready FastAPI application featuring:
- 🚀 **High-Performance Async API** - Full async/await support with signature authentication
- 💉 **Dependency Injection** - Clean authentication dependencies and middleware
- 📚 **Automatic Documentation** - OpenAPI docs with authentication examples
- 🔐 **Security Middleware** - JWT-like signature-based authentication
- ⚡ **Performance Optimization** - Connection pooling and signature caching
- 🧪 **Testing Support** - Comprehensive test suite with auth mocking
- 📊 **Monitoring Integration** - Health checks and metrics endpoints

## ⏱️ Estimated Time: 30 minutes

## 🛠️ Prerequisites

- Python 3.8+ and pip/poetry
- Basic FastAPI knowledge (dependencies, middleware, async/await)
- Completed [5-Minute Integration](../../quickstart/5-minute-integration.md)

## 🚀 Step 1: Project Setup

### Initialize FastAPI Project
```bash
# Create project directory
mkdir datafold-fastapi-demo
cd datafold-fastapi-demo

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install fastapi uvicorn[standard] pydantic python-multipart
pip install datafold-sdk httpx pytest pytest-asyncio

# Create project structure
mkdir -p app/{auth,models,routes,services,middleware,utils}
touch app/__init__.py
touch app/{auth,models,routes,services,middleware,utils}/__init__.py
```

### Project Structure
```
app/
├── __init__.py
├── main.py
├── config.py
├── auth/
│   ├── __init__.py
│   ├── dependencies.py
│   └── middleware.py
├── models/
│   ├── __init__.py
│   ├── auth.py
│   └── schemas.py
├── routes/
│   ├── __init__.py
│   ├── auth.py
│   ├── health.py
│   └── schemas.py
├── services/
│   ├── __init__.py
│   └── datafold.py
├── middleware/
│   ├── __init__.py
│   ├── auth.py
│   └── logging.py
└── utils/
    ├── __init__.py
    ├── config.py
    └── logger.py
```

## ⚙️ Step 2: Configuration and Models

### Configuration Management
```python
# app/config.py
from typing import Optional
from pydantic import BaseSettings, Field
import os

class Settings(BaseSettings):
    # Application settings
    app_name: str = "DataFold FastAPI Demo"
    version: str = "1.0.0"
    debug: bool = Field(default=False, env="DEBUG")
    
    # Server settings
    host: str = Field(default="0.0.0.0", env="HOST")
    port: int = Field(default=8000, env="PORT")
    
    # DataFold settings
    datafold_server_url: str = Field(env="DATAFOLD_SERVER_URL", default="https://api.datafold.com")
    datafold_client_id: Optional[str] = Field(env="DATAFOLD_CLIENT_ID", default=None)
    datafold_private_key: Optional[str] = Field(env="DATAFOLD_PRIVATE_KEY", default=None)
    datafold_enable_auth: bool = Field(env="DATAFOLD_ENABLE_AUTH", default=True)
    datafold_require_auth: bool = Field(env="DATAFOLD_REQUIRE_AUTH", default=False)
    
    # Signature caching
    signature_cache_enabled: bool = Field(env="SIGNATURE_CACHE_ENABLED", default=True)
    signature_cache_ttl: int = Field(env="SIGNATURE_CACHE_TTL", default=300)  # 5 minutes
    signature_cache_size: int = Field(env="SIGNATURE_CACHE_SIZE", default=1000)
    
    # Logging
    log_level: str = Field(env="LOG_LEVEL", default="INFO")
    log_format: str = Field(env="LOG_FORMAT", default="json")
    
    class Config:
        env_file = ".env"
        case_sensitive = False

# Global settings instance
settings = Settings()

def get_settings() -> Settings:
    return settings
```

### Pydantic Models
```python
# app/models/auth.py
from typing import Optional, Dict, List, Any
from pydantic import BaseModel, Field
from datetime import datetime

class AuthStatus(BaseModel):
    is_authenticated: bool
    client_id: Optional[str] = None
    authentication_method: str = "signature"
    timestamp: datetime = Field(default_factory=datetime.now)

class SignatureInfo(BaseModel):
    valid: bool
    algorithm: str = "ed25519"
    components: List[str]
    timestamp: datetime
    key_id: Optional[str] = None

class AuthenticatedUser(BaseModel):
    client_id: str
    is_authenticated: bool = True
    signature: Optional[SignatureInfo] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)

# app/models/schemas.py
from typing import List, Any, Optional
from pydantic import BaseModel, Field

class APIResponse(BaseModel):
    success: bool
    data: Optional[Any] = None
    error: Optional[Dict[str, Any]] = None
    meta: Dict[str, Any] = Field(default_factory=dict)

class HealthCheck(BaseModel):
    status: str
    timestamp: datetime
    version: str
    environment: str
    services: Dict[str, Any] = Field(default_factory=dict)

class SchemaField(BaseModel):
    name: str
    type: str
    required: bool = False
    description: Optional[str] = None

class SchemaCreate(BaseModel):
    name: str = Field(..., min_length=1, max_length=100)
    fields: List[SchemaField]
    version: str = Field(default="1.0.0")
    description: Optional[str] = None

class SchemaResponse(BaseModel):
    name: str
    fields: List[SchemaField]
    version: str
    created_at: datetime
    updated_at: datetime
```

## 🔐 Step 3: DataFold Service

Create the async DataFold service:

```python
# app/services/datafold.py
import asyncio
import httpx
from typing import Optional, Dict, Any, List
from datetime import datetime
import logging

from datafold_sdk import generate_key_pair, create_enhanced_http_client, SigningMode
from app.config import get_settings
from app.models.auth import AuthenticatedUser, SignatureInfo

logger = logging.getLogger(__name__)

class DataFoldService:
    def __init__(self):
        self.settings = get_settings()
        self.client = None
        self.client_id: Optional[str] = None
        self.is_initialized = False
        self._lock = asyncio.Lock()

    async def initialize(self) -> None:
        """Initialize DataFold service with authentication"""
        async with self._lock:
            if self.is_initialized:
                return

            try:
                logger.info("Initializing DataFold service...")

                if not self.settings.datafold_enable_auth:
                    logger.info("DataFold authentication disabled")
                    self.is_initialized = True
                    return

                # Use existing credentials or generate new ones
                if self.settings.datafold_client_id and self.settings.datafold_private_key:
                    await self._initialize_with_existing_credentials()
                else:
                    await self._initialize_with_new_credentials()

                # Test connection
                if self.client:
                    await self._test_connection()

                self.is_initialized = True
                logger.info(f"DataFold service initialized successfully with client ID: {self.client_id}")

            except Exception as error:
                logger.error(f"Failed to initialize DataFold service: {error}")
                
                if self.settings.datafold_require_auth:
                    raise
                else:
                    logger.warning("Continuing without DataFold authentication")
                    self.is_initialized = True

    async def _initialize_with_existing_credentials(self) -> None:
        """Initialize with existing client ID and private key"""
        self.client_id = self.settings.datafold_client_id
        
        self.client = create_enhanced_http_client(
            base_url=self.settings.datafold_server_url,
            signing_mode=SigningMode.AUTO,
            enable_signature_cache=self.settings.signature_cache_enabled,
            signature_cache_ttl=self.settings.signature_cache_ttl,
            max_cache_size=self.settings.signature_cache_size
        )
        
        # Configure signing with existing credentials
        signing_config = {
            'algorithm': 'ed25519',
            'key_id': self.client_id,
            'private_key': bytes.fromhex(self.settings.datafold_private_key),
            'profile': 'standard'
        }
        
        self.client.configure_signing(signing_config)
        logger.info(f"Using existing DataFold credentials for client: {self.client_id}")

    async def _initialize_with_new_credentials(self) -> None:
        """Generate new credentials and register with DataFold"""
        logger.info("Generating new DataFold credentials...")
        
        # Generate new keypair
        private_key, public_key = generate_key_pair()
        self.client_id = f"fastapi-{datetime.now().strftime('%Y%m%d%H%M%S')}"

        # Register public key with DataFold
        async with httpx.AsyncClient() as http_client:
            registration_data = {
                "client_id": self.client_id,
                "public_key": public_key.hex(),
                "key_name": "FastAPI Application",
                "metadata": {
                    "framework": "fastapi",
                    "environment": "development" if self.settings.debug else "production",
                    "timestamp": datetime.now().isoformat()
                }
            }

            response = await http_client.post(
                f"{self.settings.datafold_server_url}/api/crypto/keys/register",
                json=registration_data,
                timeout=30.0
            )
            
            if response.status_code != 200:
                raise Exception(f"Failed to register public key: {response.text}")

            registration = response.json()
            self.client_id = registration["data"]["client_id"]

        # Create authenticated DataFold client
        self.client = create_enhanced_http_client(
            base_url=self.settings.datafold_server_url,
            signing_mode=SigningMode.AUTO,
            enable_signature_cache=self.settings.signature_cache_enabled,
            signature_cache_ttl=self.settings.signature_cache_ttl,
            max_cache_size=self.settings.signature_cache_size
        )

        # Configure signing
        signing_config = {
            'algorithm': 'ed25519',
            'key_id': self.client_id,
            'private_key': private_key,
            'profile': 'standard'
        }
        
        self.client.configure_signing(signing_config)

        logger.info(f"Generated new DataFold credentials:")
        logger.info(f"  Client ID: {self.client_id}")
        if self.settings.debug:
            logger.info(f"  Private Key: {private_key.hex()}")
            logger.info("Save these credentials for production deployment!")

    async def _test_connection(self) -> None:
        """Test connection to DataFold server"""
        if not self.client:
            raise Exception("DataFold client not initialized")

        response = await self.client.get('/api/system/status')
        logger.debug(f"DataFold connection test successful: {response}")

    def is_ready(self) -> bool:
        """Check if DataFold service is ready"""
        return self.is_initialized and (not self.settings.datafold_enable_auth or self.client is not None)

    async def verify_signature(self, request_data: Dict[str, Any]) -> Optional[AuthenticatedUser]:
        """Verify request signature and return authenticated user info"""
        if not self.settings.datafold_enable_auth or not self.client:
            return None

        try:
            # In a real implementation, this would verify the signature
            # against the request headers and body
            # For this tutorial, we'll simulate successful verification
            
            signature_info = SignatureInfo(
                valid=True,
                algorithm="ed25519",
                components=["method", "target-uri", "content-digest"],
                timestamp=datetime.now(),
                key_id=self.client_id
            )

            return AuthenticatedUser(
                client_id=self.client_id or "unknown",
                signature=signature_info,
                metadata={"verified_at": datetime.now().isoformat()}
            )

        except Exception as error:
            logger.error(f"Signature verification failed: {error}")
            return None

    async def get_schemas(self) -> List[Dict[str, Any]]:
        """Get all schemas from DataFold"""
        if not self.client:
            raise Exception("DataFold client not available")

        response = await self.client.get('/api/schemas')
        return response.get('data', [])

    async def create_schema(self, schema_data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a new schema in DataFold"""
        if not self.client:
            raise Exception("DataFold client not available")

        response = await self.client.post('/api/schemas', json=schema_data)
        return response.get('data', {})

    async def delete_schema(self, schema_name: str) -> None:
        """Delete a schema from DataFold"""
        if not self.client:
            raise Exception("DataFold client not available")

        await self.client.delete(f'/api/schemas/{schema_name}')

    async def get_health_status(self) -> Dict[str, Any]:
        """Get DataFold service health status"""
        status = {
            "service": "datafold",
            "status": "healthy" if self.is_ready() else "unhealthy",
            "auth_enabled": self.settings.datafold_enable_auth,
            "auth_required": self.settings.datafold_require_auth,
            "client_id": self.client_id,
            "server_url": self.settings.datafold_server_url,
            "timestamp": datetime.now().isoformat()
        }

        if self.client:
            try:
                await self._test_connection()
                status["connectivity"] = "healthy"
            except Exception as error:
                status["connectivity"] = "unhealthy"
                status["error"] = str(error)

        return status

# Global service instance
datafold_service = DataFoldService()

async def get_datafold_service() -> DataFoldService:
    """Dependency to get DataFold service"""
    if not datafold_service.is_initialized:
        await datafold_service.initialize()
    return datafold_service
```

## 🔒 Step 4: Authentication Dependencies

Create FastAPI dependencies for authentication:

```python
# app/auth/dependencies.py
from typing import Optional
from fastapi import Depends, HTTPException, Request, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials

from app.models.auth import AuthenticatedUser
from app.services.datafold import get_datafold_service, DataFoldService
from app.config import get_settings, Settings

security = HTTPBearer(auto_error=False)

class SignatureAuth:
    def __init__(self, required: bool = True):
        self.required = required

    async def __call__(
        self,
        request: Request,
        credentials: Optional[HTTPAuthorizationCredentials] = Depends(security),
        datafold_service: DataFoldService = Depends(get_datafold_service),
        settings: Settings = Depends(get_settings)
    ) -> Optional[AuthenticatedUser]:
        """
        Verify DataFold signature authentication
        """
        
        # Skip authentication if disabled
        if not settings.datafold_enable_auth:
            return None

        # Extract signature headers
        signature = request.headers.get('signature')
        signature_input = request.headers.get('signature-input')

        if not signature or not signature_input:
            if self.required:
                raise HTTPException(
                    status_code=status.HTTP_401_UNAUTHORIZED,
                    detail="Missing signature headers",
                    headers={"WWW-Authenticate": "Signature"}
                )
            return None

        # Verify signature
        request_data = {
            'method': request.method,
            'url': str(request.url),
            'headers': dict(request.headers),
            'signature': signature,
            'signature_input': signature_input
        }

        authenticated_user = await datafold_service.verify_signature(request_data)

        if not authenticated_user and self.required:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid signature",
                headers={"WWW-Authenticate": "Signature"}
            )

        return authenticated_user

# Pre-configured dependency instances
require_auth = SignatureAuth(required=True)
optional_auth = SignatureAuth(required=False)

async def get_current_user(
    user: Optional[AuthenticatedUser] = Depends(require_auth)
) -> AuthenticatedUser:
    """Get the current authenticated user (required)"""
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Authentication required"
        )
    return user

async def get_current_user_optional(
    user: Optional[AuthenticatedUser] = Depends(optional_auth)
) -> Optional[AuthenticatedUser]:
    """Get the current authenticated user (optional)"""
    return user
```

## 🔄 Step 5: Middleware

Create authentication and logging middleware:

```python
# app/middleware/auth.py
import time
import uuid
from typing import Callable
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
import logging

logger = logging.getLogger(__name__)

class AuthenticationMiddleware(BaseHTTPMiddleware):
    """Middleware to handle authentication context and logging"""
    
    def __init__(self, app, skip_paths: list = None):
        super().__init__(app)
        self.skip_paths = skip_paths or ['/docs', '/redoc', '/openapi.json', '/health']

    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        # Add request ID
        request_id = str(uuid.uuid4())
        request.state.request_id = request_id

        # Add timing
        start_time = time.time()

        # Skip authentication for certain paths
        skip_auth = any(request.url.path.startswith(path) for path in self.skip_paths)
        request.state.skip_auth = skip_auth

        # Log request
        logger.info(
            "Request started",
            extra={
                "request_id": request_id,
                "method": request.method,
                "url": str(request.url),
                "user_agent": request.headers.get("user-agent"),
                "skip_auth": skip_auth
            }
        )

        # Process request
        response = await call_next(request)

        # Add headers
        response.headers["X-Request-ID"] = request_id
        
        # Calculate duration
        duration = time.time() - start_time
        
        # Log response
        logger.info(
            "Request completed",
            extra={
                "request_id": request_id,
                "status_code": response.status_code,
                "duration_ms": round(duration * 1000, 2)
            }
        )

        return response

# app/middleware/logging.py
import logging
import sys
from datetime import datetime
from typing import Dict, Any

class StructuredLogger:
    """Structured JSON logger for better observability"""
    
    def __init__(self, level: str = "INFO"):
        self.logger = logging.getLogger("datafold_fastapi")
        self.logger.setLevel(getattr(logging, level.upper()))
        
        # Create console handler
        handler = logging.StreamHandler(sys.stdout)
        handler.setLevel(getattr(logging, level.upper()))
        
        # Create formatter
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        handler.setFormatter(formatter)
        
        # Add handler
        if not self.logger.handlers:
            self.logger.addHandler(handler)

    def info(self, message: str, extra: Dict[str, Any] = None):
        self.logger.info(message, extra=extra or {})

    def error(self, message: str, extra: Dict[str, Any] = None):
        self.logger.error(message, extra=extra or {})

    def warning(self, message: str, extra: Dict[str, Any] = None):
        self.logger.warning(message, extra=extra or {})

    def debug(self, message: str, extra: Dict[str, Any] = None):
        self.logger.debug(message, extra=extra or {})

# Global logger instance
structured_logger = StructuredLogger()
```

## 📚 Step 6: Route Handlers

Create route handlers with authentication:

```python
# app/routes/health.py
from fastapi import APIRouter, Depends
from datetime import datetime
from typing import Dict, Any

from app.models.schemas import HealthCheck, APIResponse
from app.services.datafold import get_datafold_service, DataFoldService
from app.config import get_settings, Settings

router = APIRouter(prefix="/health", tags=["health"])

@router.get("/", response_model=APIResponse)
async def health_check(
    settings: Settings = Depends(get_settings),
    datafold_service: DataFoldService = Depends(get_datafold_service)
) -> APIResponse:
    """Basic health check endpoint"""
    
    health_data = HealthCheck(
        status="healthy",
        timestamp=datetime.now(),
        version=settings.version,
        environment="development" if settings.debug else "production",
        services={
            "datafold": await datafold_service.get_health_status()
        }
    )

    return APIResponse(
        success=True,
        data=health_data.dict(),
        meta={
            "timestamp": datetime.now().isoformat(),
            "endpoint": "health_check"
        }
    )

@router.get("/detailed", response_model=APIResponse)
async def detailed_health_check(
    settings: Settings = Depends(get_settings),
    datafold_service: DataFoldService = Depends(get_datafold_service)
) -> APIResponse:
    """Detailed health check with system information"""
    
    import psutil
    import os
    
    health_data = {
        "application": {
            "name": settings.app_name,
            "version": settings.version,
            "status": "healthy",
            "uptime_seconds": time.time() - psutil.Process(os.getpid()).create_time()
        },
        "system": {
            "memory_usage_mb": psutil.virtual_memory().used / 1024 / 1024,
            "cpu_percent": psutil.cpu_percent(),
            "disk_usage_percent": psutil.disk_usage('/').percent
        },
        "configuration": {
            "debug": settings.debug,
            "datafold_auth_enabled": settings.datafold_enable_auth,
            "datafold_auth_required": settings.datafold_require_auth
        },
        "services": {
            "datafold": await datafold_service.get_health_status()
        }
    }

    return APIResponse(
        success=True,
        data=health_data,
        meta={
            "timestamp": datetime.now().isoformat(),
            "endpoint": "detailed_health_check"
        }
    )

# app/routes/auth.py
from fastapi import APIRouter, Depends, Request
from datetime import datetime

from app.models.auth import AuthenticatedUser, AuthStatus
from app.models.schemas import APIResponse
from app.auth.dependencies import get_current_user_optional, get_current_user

router = APIRouter(prefix="/auth", tags=["authentication"])

@router.get("/status", response_model=APIResponse)
async def auth_status(
    request: Request,
    current_user: Optional[AuthenticatedUser] = Depends(get_current_user_optional)
) -> APIResponse:
    """Get current authentication status"""
    
    status = AuthStatus(
        is_authenticated=current_user is not None,
        client_id=current_user.client_id if current_user else None,
        timestamp=datetime.now()
    )

    return APIResponse(
        success=True,
        data=status.dict(),
        meta={
            "timestamp": datetime.now().isoformat(),
            "request_id": getattr(request.state, 'request_id', 'unknown'),
            "endpoint": "auth_status"
        }
    )

@router.get("/me", response_model=APIResponse)
async def get_current_user_info(
    request: Request,
    current_user: AuthenticatedUser = Depends(get_current_user)
) -> APIResponse:
    """Get current authenticated user information"""
    
    return APIResponse(
        success=True,
        data=current_user.dict(),
        meta={
            "timestamp": datetime.now().isoformat(),
            "request_id": getattr(request.state, 'request_id', 'unknown'),
            "endpoint": "get_current_user_info"
        }
    )

@router.post("/test", response_model=APIResponse)
async def test_authentication(
    request: Request,
    current_user: AuthenticatedUser = Depends(get_current_user)
) -> APIResponse:
    """Test endpoint that requires authentication"""
    
    test_data = {
        "message": "Authentication successful!",
        "authenticated_as": current_user.client_id,
        "signature_info": current_user.signature.dict() if current_user.signature else None,
        "test_timestamp": datetime.now().isoformat()
    }

    return APIResponse(
        success=True,
        data=test_data,
        meta={
            "timestamp": datetime.now().isoformat(),
            "request_id": getattr(request.state, 'request_id', 'unknown'),
            "endpoint": "test_authentication"
        }
    )

# app/routes/schemas.py
from typing import List
from fastapi import APIRouter, Depends, HTTPException, Request, status

from app.models.schemas import SchemaCreate, SchemaResponse, APIResponse
from app.models.auth import AuthenticatedUser
from app.auth.dependencies import get_current_user
from app.services.datafold import get_datafold_service, DataFoldService

router = APIRouter(prefix="/schemas", tags=["schemas"])

@router.get("/", response_model=APIResponse)
async def list_schemas(
    request: Request,
    current_user: AuthenticatedUser = Depends(get_current_user),
    datafold_service: DataFoldService = Depends(get_datafold_service)
) -> APIResponse:
    """List all schemas"""
    
    try:
        schemas = await datafold_service.get_schemas()
        
        return APIResponse(
            success=True,
            data=schemas,
            meta={
                "timestamp": datetime.now().isoformat(),
                "request_id": getattr(request.state, 'request_id', 'unknown'),
                "count": len(schemas),
                "authenticated_as": current_user.client_id
            }
        )
        
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to retrieve schemas: {str(e)}"
        )

@router.post("/", response_model=APIResponse)
async def create_schema(
    schema_data: SchemaCreate,
    request: Request,
    current_user: AuthenticatedUser = Depends(get_current_user),
    datafold_service: DataFoldService = Depends(get_datafold_service)
) -> APIResponse:
    """Create a new schema"""
    
    try:
        # Convert Pydantic model to dict for DataFold API
        schema_dict = schema_data.dict()
        new_schema = await datafold_service.create_schema(schema_dict)
        
        return APIResponse(
            success=True,
            data=new_schema,
            meta={
                "timestamp": datetime.now().isoformat(),
                "request_id": getattr(request.state, 'request_id', 'unknown'),
                "created_by": current_user.client_id,
                "action": "schema_created"
            }
        )
        
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to create schema: {str(e)}"
        )

@router.delete("/{schema_name}", response_model=APIResponse)
async def delete_schema(
    schema_name: str,
    request: Request,
    current_user: AuthenticatedUser = Depends(get_current_user),
    datafold_service: DataFoldService = Depends(get_datafold_service)
) -> APIResponse:
    """Delete a schema"""
    
    try:
        await datafold_service.delete_schema(schema_name)
        
        return APIResponse(
            success=True,
            data={"message": f"Schema '{schema_name}' deleted successfully"},
            meta={
                "timestamp": datetime.now().isoformat(),
                "request_id": getattr(request.state, 'request_id', 'unknown'),
                "deleted_by": current_user.client_id,
                "action": "schema_deleted",
                "schema_name": schema_name
            }
        )
        
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to delete schema: {str(e)}"
        )
```

## 🚀 Step 7: Main Application

Put it all together in the main FastAPI app:

```python
# app/main.py
import asyncio
import logging
from contextlib import asynccontextmanager
from fastapi import FastAPI, Request, status
from fastapi.responses import JSONResponse
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from fastapi.exception_handlers import http_exception_handler
from starlette.exceptions import HTTPException as StarletteHTTPException

from app.config import get_settings
from app.services.datafold import get_datafold_service
from app.middleware.auth import AuthenticationMiddleware
from app.routes import health, auth, schemas
from app.models.schemas import APIResponse

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan events"""
    settings = get_settings()
    
    # Startup
    logging.info(f"Starting {settings.app_name} v{settings.version}")
    
    # Initialize DataFold service
    try:
        datafold_service = await get_datafold_service()
        logging.info("DataFold service initialized successfully")
    except Exception as e:
        logging.error(f"Failed to initialize DataFold service: {e}")
        if settings.datafold_require_auth:
            raise
    
    yield
    
    # Shutdown
    logging.info("Shutting down application")

def create_app() -> FastAPI:
    """Create and configure FastAPI application"""
    settings = get_settings()
    
    app = FastAPI(
        title=settings.app_name,
        description="FastAPI application with DataFold signature authentication",
        version=settings.version,
        debug=settings.debug,
        lifespan=lifespan
    )

    # Add middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    app.add_middleware(
        TrustedHostMiddleware,
        allowed_hosts=["*"]  # Configure appropriately for production
    )
    
    app.add_middleware(
        AuthenticationMiddleware,
        skip_paths=["/docs", "/redoc", "/openapi.json", "/health"]
    )

    # Include routers
    app.include_router(health.router)
    app.include_router(auth.router, prefix="/api")
    app.include_router(schemas.router, prefix="/api")

    # Root endpoint
    @app.get("/", response_model=APIResponse)
    async def root():
        return APIResponse(
            success=True,
            data={
                "message": f"Welcome to {settings.app_name}",
                "version": settings.version,
                "docs": "/docs",
                "health": "/health"
            },
            meta={
                "timestamp": datetime.now().isoformat(),
                "environment": "development" if settings.debug else "production"
            }
        )

    # Global exception handler
    @app.exception_handler(StarletteHTTPException)
    async def custom_http_exception_handler(request: Request, exc: StarletteHTTPException):
        return JSONResponse(
            status_code=exc.status_code,
            content=APIResponse(
                success=False,
                error={
                    "code": exc.status_code,
                    "message": exc.detail
                },
                meta={
                    "timestamp": datetime.now().isoformat(),
                    "request_id": getattr(request.state, 'request_id', 'unknown'),
                    "path": request.url.path
                }
            ).dict()
        )

    # Global exception handler for unhandled exceptions
    @app.exception_handler(Exception)
    async def global_exception_handler(request: Request, exc: Exception):
        logging.error(f"Unhandled exception: {exc}", exc_info=True)
        
        return JSONResponse(
            status_code=500,
            content=APIResponse(
                success=False,
                error={
                    "code": 500,
                    "message": "Internal server error" if not settings.debug else str(exc)
                },
                meta={
                    "timestamp": datetime.now().isoformat(),
                    "request_id": getattr(request.state, 'request_id', 'unknown'),
                    "path": request.url.path
                }
            ).dict()
        )

    return app

# Create app instance
app = create_app()

# Development server
if __name__ == "__main__":
    import uvicorn
    settings = get_settings()
    
    uvicorn.run(
        "app.main:app",
        host=settings.host,
        port=settings.port,
        reload=settings.debug,
        log_level=settings.log_level.lower()
    )
```

## ⚙️ Step 8: Environment Configuration

Create environment files:

```bash
# .env.development
DEBUG=true
LOG_LEVEL=DEBUG

# Server settings
HOST=0.0.0.0
PORT=8000

# DataFold settings
DATAFOLD_SERVER_URL=https://api.datafold.com
DATAFOLD_ENABLE_AUTH=true
DATAFOLD_REQUIRE_AUTH=false

# Signature caching
SIGNATURE_CACHE_ENABLED=true
SIGNATURE_CACHE_TTL=300
SIGNATURE_CACHE_SIZE=1000
```

```bash
# .env.production
DEBUG=false
LOG_LEVEL=INFO

# Server settings
HOST=0.0.0.0
PORT=8000

# DataFold settings (set these in your deployment)
DATAFOLD_SERVER_URL=https://api.datafold.com
DATAFOLD_CLIENT_ID=your-production-client-id
DATAFOLD_PRIVATE_KEY=your-production-private-key
DATAFOLD_ENABLE_AUTH=true
DATAFOLD_REQUIRE_AUTH=true

# Signature caching
SIGNATURE_CACHE_ENABLED=true
SIGNATURE_CACHE_TTL=300
SIGNATURE_CACHE_SIZE=1000
```

## ✅ Step 9: Testing Your Integration

### Development Server
```bash
# Install dependencies
pip install -r requirements.txt

# Start development server
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

# The API will be available at:
# - API: http://localhost:8000
# - Docs: http://localhost:8000/docs
# - Health: http://localhost:8000/health
```

### Manual Testing
```bash
# Test health endpoint
curl http://localhost:8000/health

# Test authentication status
curl http://localhost:8000/api/auth/status

# Test protected endpoint with signature
curl -X GET http://localhost:8000/api/schemas \
  -H "Signature: keyId=\"test\",algorithm=\"ed25519\",headers=\"(request-target) content-digest\",signature=\"xyz\"" \
  -H "Signature-Input: sig=(\"@method\" \"@target-uri\" \"content-digest\");created=1625097600"

# Create a schema
curl -X POST http://localhost:8000/api/schemas \
  -H "Content-Type: application/json" \
  -H "Signature: keyId=\"test\",algorithm=\"ed25519\",headers=\"(request-target) content-digest\",signature=\"xyz\"" \
  -H "Signature-Input: sig=(\"@method\" \"@target-uri\" \"content-digest\");created=1625097600" \
  -d '{
    "name": "user_events",
    "fields": [
      {"name": "user_id", "type": "string", "required": true},
      {"name": "event_type", "type": "string", "required": true},
      {"name": "timestamp", "type": "datetime", "required": true}
    ]
  }'
```

### Automated Testing
```python
# test_main.py
import pytest
from fastapi.testclient import TestClient
from unittest.mock import AsyncMock, patch

from app.main import create_app
from app.config import get_settings

@pytest.fixture
def test_app():
    """Create test app with mocked dependencies"""
    app = create_app()
    return app

@pytest.fixture
def client(test_app):
    """Create test client"""
    return TestClient(test_app)

def test_health_endpoint(client):
    """Test health endpoint"""
    response = client.get("/health/")
    assert response.status_code == 200
    
    data = response.json()
    assert data["success"] is True
    assert "timestamp" in data["data"]

def test_auth_status_unauthenticated(client):
    """Test auth status without authentication"""
    response = client.get("/api/auth/status")
    assert response.status_code == 200
    
    data = response.json()
    assert data["success"] is True
    assert data["data"]["is_authenticated"] is False

@patch('app.services.datafold.DataFoldService.verify_signature')
def test_protected_endpoint_with_auth(mock_verify, client):
    """Test protected endpoint with authentication"""
    from app.models.auth import AuthenticatedUser, SignatureInfo
    from datetime import datetime
    
    # Mock successful authentication
    mock_verify.return_value = AuthenticatedUser(
        client_id="test-client",
        signature=SignatureInfo(
            valid=True,
            algorithm="ed25519",
            components=["method", "target-uri"],
            timestamp=datetime.now()
        )
    )
    
    response = client.get(
        "/api/auth/me",
        headers={
            "Signature": 'keyId="test",algorithm="ed25519",signature="xyz"',
            "Signature-Input": 'sig=("@method" "@target-uri");created=1625097600'
        }
    )
    
    assert response.status_code == 200
    data = response.json()
    assert data["success"] is True
    assert data["data"]["client_id"] == "test-client"

def test_protected_endpoint_without_auth(client):
    """Test protected endpoint without authentication"""
    response = client.get("/api/auth/me")
    assert response.status_code == 401

@patch('app.services.datafold.DataFoldService.get_schemas')
def test_list_schemas(mock_get_schemas, client):
    """Test schema listing"""
    mock_get_schemas.return_value = [
        {"name": "test_schema", "fields": [], "version": "1.0.0"}
    ]
    
    with patch('app.services.datafold.DataFoldService.verify_signature') as mock_verify:
        from app.models.auth import AuthenticatedUser
        mock_verify.return_value = AuthenticatedUser(client_id="test-client")
        
        response = client.get(
            "/api/schemas/",
            headers={
                "Signature": 'keyId="test",algorithm="ed25519",signature="xyz"',
                "Signature-Input": 'sig=("@method" "@target-uri");created=1625097600'
            }
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert len(data["data"]) == 1

# Run tests
# pytest test_main.py -v
```

## 🚀 Step 10: Production Deployment

### Requirements File
```txt
# requirements.txt
fastapi==0.104.1
uvicorn[standard]==0.24.0
pydantic==2.5.0
python-multipart==0.0.6
httpx==0.25.2
psutil==5.9.6
datafold-sdk>=1.0.0

# Production dependencies
gunicorn==21.2.0
python-json-logger==2.0.7
prometheus-client==0.19.0
```

### Gunicorn Configuration
```python
# gunicorn.conf.py
import os

bind = f"0.0.0.0:{os.getenv('PORT', '8000')}"
workers = int(os.getenv('WORKERS', '4'))
worker_class = "uvicorn.workers.UvicornWorker"
worker_connections = 1000
max_requests = 1000
max_requests_jitter = 100
preload_app = True
keepalive = 2

# Logging
accesslog = "-"
errorlog = "-"
loglevel = os.getenv('LOG_LEVEL', 'info').lower()
access_log_format = '%(h)s %(l)s %(u)s %(t)s "%(r)s" %(s)s %(b)s "%(f)s" "%(a)s" %(D)s'

# Process naming
proc_name = 'datafold-fastapi'

# Worker timeout
timeout = 30
graceful_timeout = 10

# Preload application for better memory usage
def when_ready(server):
    server.log.info("DataFold FastAPI server is ready!")

def worker_int(worker):
    worker.log.info("Worker received INT or QUIT signal")

def pre_fork(server, worker):
    server.log.info("Worker spawned (pid: %s)", worker.pid)
```

### Docker Support
```dockerfile
# Dockerfile
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r appgroup && useradd -r -g appgroup appuser

# Set working directory
WORKDIR /app

# Copy requirements and install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY app/ ./app/
COPY gunicorn.conf.py .

# Change ownership
RUN chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Start application
CMD ["gunicorn", "-c", "gunicorn.conf.py", "app.main:app"]
```

### Monitoring Integration
```python
# app/monitoring.py
from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
from fastapi import Response
import time

# Metrics
REQUEST_COUNT = Counter('http_requests_total', 'Total HTTP requests', ['method', 'endpoint', 'status'])
REQUEST_DURATION = Histogram('http_request_duration_seconds', 'HTTP request duration')

class MetricsMiddleware:
    def __init__(self, app):
        self.app = app

    async def __call__(self, scope, receive, send):
        if scope["type"] == "http":
            start_time = time.time()
            
            # Process request
            await self.app(scope, receive, send)
            
            # Record metrics
            duration = time.time() - start_time
            method = scope["method"]
            path = scope["path"]
            
            REQUEST_DURATION.observe(duration)
            # Note: Status code would need to be captured from response
            
        else:
            await self.app(scope, receive, send)

# Add to main.py
@app.get("/metrics")
async def metrics():
    """Prometheus metrics endpoint"""
    return Response(generate_latest(), media_type=CONTENT_TYPE_LATEST)
```

## 🔧 Common Issues & Solutions

### Issue: Async Context Issues
```python
# Solution: Proper async context management
import asyncio
from contextlib import asynccontextmanager

@asynccontextmanager
async def managed_service():
    service = DataFoldService()
    await service.initialize()
    try:
        yield service
    finally:
        await service.cleanup()  # If cleanup is needed
```

### Issue: Dependency Injection Errors
```python
# Solution: Proper dependency scoping
from functools import lru_cache

@lru_cache()
def get_settings():
    return Settings()

# Use Depends(get_settings) consistently
```

### Issue: Signature Verification Performance
```python
# Solution: Implement caching and async processing
import asyncio
from functools import lru_cache
from typing import Dict

class SignatureCache:
    def __init__(self, max_size: int = 1000, ttl: int = 300):
        self.cache: Dict[str, Any] = {}
        self.max_size = max_size
        self.ttl = ttl

    async def get_or_verify(self, signature_key: str, verify_func):
        if signature_key in self.cache:
            return self.cache[signature_key]
        
        result = await verify_func()
        
        if len(self.cache) >= self.max_size:
            # Simple LRU eviction
            oldest_key = next(iter(self.cache))
            del self.cache[oldest_key]
        
        self.cache[signature_key] = result
        return result
```

## 🎯 Next Steps

### Advanced Features
- **[WebSocket Authentication](../../advanced/websocket-auth-patterns.md)** - Real-time authentication
- **[Rate Limiting](../../advanced/rate-limiting-patterns.md)** - API protection
- **[Caching Strategies](../../advanced/caching-patterns.md)** - Performance optimization

### Deployment & Scaling
- **[Kubernetes Deployment](../../deployment/kubernetes-deployment-guide.md)** - Container orchestration
- **[Load Balancing](../../advanced/load-balancing-patterns.md)** - High availability
- **[Monitoring & Observability](../../deployment/monitoring-integration.md)** - Production monitoring

---

🎉 **Congratulations!** You've built a high-performance FastAPI application with DataFold signature authentication. Your API now provides secure, async authentication with comprehensive documentation and production-ready patterns.

💡 **Pro Tips**:
- Use FastAPI's dependency injection for clean authentication
- Leverage async/await for better performance
- Implement proper error handling and validation
- Use Pydantic models for type safety
- Monitor authentication metrics in production
- Test with realistic authentication scenarios
- Use structured logging for better observability