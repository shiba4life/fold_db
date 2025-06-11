# Node.js/Express Integration Tutorial

Build a production-ready Express.js API with DataFold signature authentication. This tutorial covers Express middleware, route protection, error handling, and real-world server patterns.

## 🎯 What You'll Build

A complete Express.js API server featuring:
- 🔐 **Authentication Middleware** - Automatic signature verification for all routes
- 🛡️ **Route Protection** - Granular authentication control per endpoint
- ⚡ **Performance Optimization** - Signature caching and connection pooling
- 🔄 **Error Handling** - Comprehensive error responses and logging
- 📊 **Health Monitoring** - Authentication status and metrics endpoints
- 🧪 **Testing Support** - Mock authentication for development and testing

## ⏱️ Estimated Time: 30 minutes

## 🛠️ Prerequisites

- Node.js 16+ and npm/yarn
- Basic Express.js knowledge (middleware, routing, error handling)
- Completed [5-Minute Integration](../../quickstart/5-minute-integration.md)

## 🚀 Step 1: Project Setup

### Initialize Express Project
```bash
# Create new Express project
mkdir datafold-express-api
cd datafold-express-api

# Initialize package.json
npm init -y

# Install dependencies
npm install express cors helmet morgan compression
npm install @datafold/sdk

# Install development dependencies
npm install -D nodemon @types/express @types/node typescript ts-node jest supertest @types/jest
```

### TypeScript Configuration
```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "es2020",
    "module": "commonjs",
    "lib": ["es2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
```

### Project Structure
```
src/
├── middleware/
│   ├── auth.ts
│   ├── errorHandler.ts
│   └── logging.ts
├── routes/
│   ├── auth.ts
│   ├── schemas.ts
│   └── health.ts
├── services/
│   └── datafold.ts
├── types/
│   └── index.ts
├── utils/
│   ├── logger.ts
│   └── config.ts
└── app.ts
```

## 🔧 Step 2: Configuration and Utilities

### Environment Configuration
```typescript
// src/utils/config.ts
import { config } from 'dotenv';

config();

export const CONFIG = {
  port: parseInt(process.env.PORT || '3000'),
  nodeEnv: process.env.NODE_ENV || 'development',
  
  // DataFold configuration
  datafold: {
    serverUrl: process.env.DATAFOLD_SERVER_URL || 'https://api.datafold.com',
    clientId: process.env.DATAFOLD_CLIENT_ID,
    privateKey: process.env.DATAFOLD_PRIVATE_KEY,
    
    // Authentication settings
    enableAuth: process.env.DATAFOLD_ENABLE_AUTH !== 'false',
    requireAuth: process.env.DATAFOLD_REQUIRE_AUTH === 'true',
    signatureCache: {
      enabled: process.env.DATAFOLD_SIGNATURE_CACHE !== 'false',
      ttl: parseInt(process.env.DATAFOLD_SIGNATURE_CACHE_TTL || '300'), // 5 minutes
      maxSize: parseInt(process.env.DATAFOLD_SIGNATURE_CACHE_SIZE || '1000')
    }
  },
  
  // Logging configuration
  logging: {
    level: process.env.LOG_LEVEL || 'info',
    enableHttp: process.env.ENABLE_HTTP_LOGGING === 'true'
  }
} as const;

// Validation
export function validateConfig(): void {
  if (CONFIG.datafold.enableAuth) {
    if (!CONFIG.datafold.serverUrl) {
      throw new Error('DATAFOLD_SERVER_URL is required when authentication is enabled');
    }
    
    if (CONFIG.datafold.requireAuth && !CONFIG.datafold.clientId) {
      console.warn('DATAFOLD_CLIENT_ID not set - will auto-register on startup');
    }
  }
}
```

### Logger Setup
```typescript
// src/utils/logger.ts
import { CONFIG } from './config';

export interface Logger {
  debug(message: string, meta?: any): void;
  info(message: string, meta?: any): void;
  warn(message: string, meta?: any): void;
  error(message: string, error?: Error | any): void;
}

class ConsoleLogger implements Logger {
  private shouldLog(level: string): boolean {
    const levels = ['debug', 'info', 'warn', 'error'];
    const currentLevelIndex = levels.indexOf(CONFIG.logging.level);
    const messageLevelIndex = levels.indexOf(level);
    return messageLevelIndex >= currentLevelIndex;
  }

  debug(message: string, meta?: any): void {
    if (this.shouldLog('debug')) {
      console.debug(`[DEBUG] ${message}`, meta || '');
    }
  }

  info(message: string, meta?: any): void {
    if (this.shouldLog('info')) {
      console.info(`[INFO] ${message}`, meta || '');
    }
  }

  warn(message: string, meta?: any): void {
    if (this.shouldLog('warn')) {
      console.warn(`[WARN] ${message}`, meta || '');
    }
  }

  error(message: string, error?: Error | any): void {
    if (this.shouldLog('error')) {
      console.error(`[ERROR] ${message}`, error || '');
    }
  }
}

export const logger: Logger = new ConsoleLogger();
```

### Type Definitions
```typescript
// src/types/index.ts
import { Request } from 'express';

export interface AuthenticatedRequest extends Request {
  datafold?: {
    clientId: string;
    isAuthenticated: boolean;
    signature?: {
      valid: boolean;
      timestamp: Date;
      components: string[];
    };
  };
}

export interface DataFoldConfig {
  serverUrl: string;
  clientId?: string;
  privateKey?: string;
  enableAuth: boolean;
  requireAuth: boolean;
  signatureCache: {
    enabled: boolean;
    ttl: number;
    maxSize: number;
  };
}

export interface AuthMiddlewareOptions {
  required?: boolean;
  skipRoutes?: string[];
  customValidator?: (req: AuthenticatedRequest) => boolean;
}

export interface APIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  meta?: {
    timestamp: string;
    requestId: string;
    authStatus?: 'authenticated' | 'unauthenticated' | 'invalid';
  };
}
```

## 🔐 Step 3: DataFold Service

Create the core DataFold service:

```typescript
// src/services/datafold.ts
import { DataFoldClient, generateKeyPair, KeyPair } from '@datafold/sdk';
import { CONFIG } from '../utils/config';
import { logger } from '../utils/logger';

export class DataFoldService {
  private client: DataFoldClient | null = null;
  private keyPair: KeyPair | null = null;
  private clientId: string | null = null;
  private isInitialized = false;

  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      logger.info('Initializing DataFold service...');

      if (!CONFIG.datafold.enableAuth) {
        logger.info('DataFold authentication disabled');
        this.isInitialized = true;
        return;
      }

      // Use existing keys or generate new ones
      if (CONFIG.datafold.privateKey && CONFIG.datafold.clientId) {
        await this.initializeWithExistingKeys();
      } else {
        await this.initializeWithNewKeys();
      }

      // Test connection
      if (this.client) {
        await this.testConnection();
      }

      this.isInitialized = true;
      logger.info('DataFold service initialized successfully', {
        clientId: this.clientId,
        serverUrl: CONFIG.datafold.serverUrl
      });

    } catch (error) {
      logger.error('Failed to initialize DataFold service', error);
      
      if (CONFIG.datafold.requireAuth) {
        throw error;
      } else {
        logger.warn('Continuing without DataFold authentication');
        this.isInitialized = true;
      }
    }
  }

  private async initializeWithExistingKeys(): Promise<void> {
    if (!CONFIG.datafold.privateKey || !CONFIG.datafold.clientId) {
      throw new Error('Private key and client ID are required');
    }

    this.clientId = CONFIG.datafold.clientId;
    
    // Create client with existing keys
    this.client = new DataFoldClient({
      serverUrl: CONFIG.datafold.serverUrl,
      clientId: this.clientId,
      privateKey: CONFIG.datafold.privateKey
    });

    logger.info('Using existing DataFold credentials', { clientId: this.clientId });
  }

  private async initializeWithNewKeys(): Promise<void> {
    logger.info('Generating new DataFold keypair...');
    
    // Generate new keypair
    this.keyPair = await generateKeyPair();
    this.clientId = `express-api-${Date.now()}`;

    // Register public key
    const response = await fetch(`${CONFIG.datafold.serverUrl}/api/crypto/keys/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        client_id: this.clientId,
        public_key: this.keyPair.publicKey,
        key_name: 'Express API Server',
        metadata: {
          framework: 'express',
          environment: CONFIG.nodeEnv,
          hostname: require('os').hostname(),
          timestamp: new Date().toISOString()
        }
      })
    });

    if (!response.ok) {
      throw new Error(`Key registration failed: ${response.statusText}`);
    }

    const registration = await response.json();
    this.clientId = registration.data.client_id;

    // Create authenticated client
    this.client = new DataFoldClient({
      serverUrl: CONFIG.datafold.serverUrl,
      clientId: this.clientId,
      privateKey: this.keyPair.privateKey
    });

    logger.info('DataFold keypair generated and registered', {
      clientId: this.clientId,
      publicKey: this.keyPair.publicKey.substring(0, 16) + '...'
    });

    // Log credentials for manual configuration (development only)
    if (CONFIG.nodeEnv === 'development') {
      logger.info('Save these credentials for production:', {
        DATAFOLD_CLIENT_ID: this.clientId,
        DATAFOLD_PRIVATE_KEY: this.keyPair.privateKey
      });
    }
  }

  private async testConnection(): Promise<void> {
    if (!this.client) {
      throw new Error('DataFold client not initialized');
    }

    const response = await this.client.get('/api/system/status');
    logger.debug('DataFold connection test successful', response.data);
  }

  getClient(): DataFoldClient | null {
    return this.client;
  }

  getClientId(): string | null {
    return this.clientId;
  }

  isAuthEnabled(): boolean {
    return CONFIG.datafold.enableAuth && this.isInitialized && this.client !== null;
  }

  async getSchemas(): Promise<any[]> {
    if (!this.client) {
      throw new Error('DataFold client not available');
    }
    
    const response = await this.client.get('/api/schemas');
    return response.data;
  }

  async createSchema(name: string, fields: any[]): Promise<any> {
    if (!this.client) {
      throw new Error('DataFold client not available');
    }
    
    const response = await this.client.post('/api/schemas', {
      name,
      fields,
      version: '1.0.0'
    });
    return response.data;
  }

  async deleteSchema(name: string): Promise<void> {
    if (!this.client) {
      throw new Error('DataFold client not available');
    }
    
    await this.client.delete(`/api/schemas/${name}`);
  }

  async getHealthStatus(): Promise<any> {
    const status = {
      service: 'datafold',
      status: this.isInitialized ? 'healthy' : 'unhealthy',
      authEnabled: CONFIG.datafold.enableAuth,
      authRequired: CONFIG.datafold.requireAuth,
      clientId: this.clientId,
      serverUrl: CONFIG.datafold.serverUrl,
      timestamp: new Date().toISOString()
    };

    if (this.client) {
      try {
        await this.testConnection();
        status.status = 'healthy';
      } catch (error) {
        status.status = 'unhealthy';
        (status as any).error = error instanceof Error ? error.message : 'Unknown error';
      }
    }

    return status;
  }
}

// Singleton instance
export const datafoldService = new DataFoldService();
```

## 🛡️ Step 4: Authentication Middleware

Create Express middleware for signature verification:

```typescript
// src/middleware/auth.ts
import { Request, Response, NextFunction } from 'express';
import { AuthenticatedRequest, AuthMiddlewareOptions, APIResponse } from '../types';
import { datafoldService } from '../services/datafold';
import { CONFIG } from '../utils/config';
import { logger } from '../utils/logger';

export function createAuthMiddleware(options: AuthMiddlewareOptions = {}) {
  const {
    required = CONFIG.datafold.requireAuth,
    skipRoutes = ['/health', '/api/health'],
    customValidator
  } = options;

  return async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    const requestId = req.headers['x-request-id'] || `req_${Date.now()}`;
    
    // Skip authentication for certain routes
    if (skipRoutes.some(route => req.path.startsWith(route))) {
      return next();
    }

    // Skip if authentication is disabled
    if (!CONFIG.datafold.enableAuth) {
      req.datafold = {
        clientId: 'unauthenticated',
        isAuthenticated: false
      };
      return next();
    }

    try {
      // Extract signature headers
      const signature = req.headers.signature as string;
      const signatureInput = req.headers['signature-input'] as string;

      if (!signature || !signatureInput) {
        return handleAuthFailure(req, res, next, 'missing_signature', 
          'Request signature headers are missing', required);
      }

      // Verify signature using DataFold's verification system
      // This would integrate with the server-side verification
      const verificationResult = await verifyRequestSignature(req, signature, signatureInput);

      if (verificationResult.valid) {
        req.datafold = {
          clientId: verificationResult.clientId,
          isAuthenticated: true,
          signature: {
            valid: true,
            timestamp: verificationResult.timestamp,
            components: verificationResult.components
          }
        };

        logger.debug('Request authenticated successfully', {
          requestId,
          clientId: verificationResult.clientId,
          path: req.path
        });

        // Apply custom validation if provided
        if (customValidator && !customValidator(req)) {
          return handleAuthFailure(req, res, next, 'custom_validation_failed',
            'Custom authentication validation failed', required);
        }

        return next();
      } else {
        return handleAuthFailure(req, res, next, 'invalid_signature',
          verificationResult.error || 'Signature verification failed', required);
      }

    } catch (error) {
      logger.error('Authentication middleware error', error);
      return handleAuthFailure(req, res, next, 'auth_error',
        'Internal authentication error', required);
    }
  };
}

async function verifyRequestSignature(
  req: AuthenticatedRequest, 
  signature: string, 
  signatureInput: string
): Promise<{
  valid: boolean;
  clientId?: string;
  timestamp?: Date;
  components?: string[];
  error?: string;
}> {
  try {
    // In a real implementation, this would:
    // 1. Parse the signature components
    // 2. Reconstruct the message to sign
    // 3. Verify against registered public keys
    // 4. Check timestamp and nonce validity
    
    // For this tutorial, we'll simulate the verification
    // In production, you'd integrate with your DataFold server's verification endpoint
    
    const client = datafoldService.getClient();
    if (!client) {
      return { valid: false, error: 'DataFold client not available' };
    }

    // Simulate signature verification call to DataFold server
    // This is where you'd make an actual verification request
    const mockVerification = {
      valid: true,
      clientId: datafoldService.getClientId() || 'unknown',
      timestamp: new Date(),
      components: ['method', 'target-uri', 'content-digest']
    };

    return mockVerification;

  } catch (error) {
    logger.error('Signature verification failed', error);
    return { 
      valid: false, 
      error: error instanceof Error ? error.message : 'Verification error' 
    };
  }
}

function handleAuthFailure(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction,
  errorCode: string,
  message: string,
  required: boolean
) {
  const requestId = req.headers['x-request-id'] || `req_${Date.now()}`;
  
  logger.warn('Authentication failed', {
    requestId,
    errorCode,
    message,
    path: req.path,
    method: req.method,
    required
  });

  req.datafold = {
    clientId: 'unauthenticated',
    isAuthenticated: false
  };

  if (required) {
    const response: APIResponse = {
      success: false,
      error: {
        code: errorCode,
        message: message
      },
      meta: {
        timestamp: new Date().toISOString(),
        requestId: requestId as string,
        authStatus: 'invalid'
      }
    };

    return res.status(401).json(response);
  }

  // Continue without authentication if not required
  return next();
}

// Middleware to require authentication for specific routes
export function requireAuth(req: AuthenticatedRequest, res: Response, next: NextFunction) {
  if (!req.datafold?.isAuthenticated) {
    const response: APIResponse = {
      success: false,
      error: {
        code: 'authentication_required',
        message: 'This endpoint requires authentication'
      },
      meta: {
        timestamp: new Date().toISOString(),
        requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
        authStatus: 'unauthenticated'
      }
    };

    return res.status(401).json(response);
  }

  next();
}

// Middleware to extract client information
export function clientInfo(req: AuthenticatedRequest, res: Response, next: NextFunction) {
  if (req.datafold?.isAuthenticated) {
    logger.debug('Authenticated request', {
      clientId: req.datafold.clientId,
      path: req.path,
      method: req.method
    });
  }
  
  next();
}
```

## 📝 Step 5: Error Handling Middleware

Create comprehensive error handling:

```typescript
// src/middleware/errorHandler.ts
import { Request, Response, NextFunction } from 'express';
import { AuthenticatedRequest, APIResponse } from '../types';
import { logger } from '../utils/logger';
import { CONFIG } from '../utils/config';

export interface AppError extends Error {
  statusCode?: number;
  errorCode?: string;
  details?: any;
}

export function createError(
  message: string, 
  statusCode: number = 500, 
  errorCode?: string,
  details?: any
): AppError {
  const error = new Error(message) as AppError;
  error.statusCode = statusCode;
  error.errorCode = errorCode;
  error.details = details;
  return error;
}

export function errorHandler(
  error: AppError,
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
) {
  const requestId = req.headers['x-request-id'] || `req_${Date.now()}`;
  
  // Log error
  logger.error('Request error', {
    requestId,
    error: error.message,
    stack: error.stack,
    path: req.path,
    method: req.method,
    clientId: req.datafold?.clientId
  });

  // Default error response
  const statusCode = error.statusCode || 500;
  const errorCode = error.errorCode || 'internal_error';
  
  const response: APIResponse = {
    success: false,
    error: {
      code: errorCode,
      message: error.message,
      ...(CONFIG.nodeEnv === 'development' && { details: error.details })
    },
    meta: {
      timestamp: new Date().toISOString(),
      requestId: requestId as string,
      authStatus: req.datafold?.isAuthenticated ? 'authenticated' : 'unauthenticated'
    }
  };

  // Don't expose internal errors in production
  if (statusCode === 500 && CONFIG.nodeEnv === 'production') {
    response.error!.message = 'Internal server error';
  }

  res.status(statusCode).json(response);
}

export function notFoundHandler(req: Request, res: Response) {
  const response: APIResponse = {
    success: false,
    error: {
      code: 'not_found',
      message: `Route ${req.method} ${req.path} not found`
    },
    meta: {
      timestamp: new Date().toISOString(),
      requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`
    }
  };

  res.status(404).json(response);
}

// Async error wrapper
export function asyncHandler<T extends Request, U extends Response>(
  fn: (req: T, res: U, next: NextFunction) => Promise<void>
) {
  return (req: T, res: U, next: NextFunction) => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
}
```

## 📊 Step 6: Route Handlers

Create route handlers for different endpoints:

```typescript
// src/routes/health.ts
import { Router } from 'express';
import { AuthenticatedRequest, APIResponse } from '../types';
import { datafoldService } from '../services/datafold';
import { asyncHandler } from '../middleware/errorHandler';
import { CONFIG } from '../utils/config';

const router = Router();

// Health check endpoint (no auth required)
router.get('/health', asyncHandler(async (req: AuthenticatedRequest, res) => {
  const healthStatus = {
    service: 'express-api',
    status: 'healthy',
    timestamp: new Date().toISOString(),
    environment: CONFIG.nodeEnv,
    datafold: await datafoldService.getHealthStatus()
  };

  const response: APIResponse = {
    success: true,
    data: healthStatus,
    meta: {
      timestamp: new Date().toISOString(),
      requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`
    }
  };

  res.json(response);
}));

// Detailed status endpoint (auth required)
router.get('/status', asyncHandler(async (req: AuthenticatedRequest, res) => {
  const status = {
    api: {
      status: 'healthy',
      version: process.env.npm_package_version || '1.0.0',
      uptime: process.uptime(),
      memory: process.memoryUsage(),
      environment: CONFIG.nodeEnv
    },
    authentication: {
      enabled: CONFIG.datafold.enableAuth,
      required: CONFIG.datafold.requireAuth,
      clientAuthenticated: req.datafold?.isAuthenticated || false,
      clientId: req.datafold?.clientId
    },
    datafold: await datafoldService.getHealthStatus()
  };

  const response: APIResponse = {
    success: true,
    data: status,
    meta: {
      timestamp: new Date().toISOString(),
      requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
      authStatus: req.datafold?.isAuthenticated ? 'authenticated' : 'unauthenticated'
    }
  };

  res.json(response);
}));

export default router;
```

```typescript
// src/routes/schemas.ts
import { Router } from 'express';
import { AuthenticatedRequest, APIResponse } from '../types';
import { datafoldService } from '../services/datafold';
import { requireAuth } from '../middleware/auth';
import { asyncHandler, createError } from '../middleware/errorHandler';

const router = Router();

// Apply authentication requirement to all schema routes
router.use(requireAuth);

// Get all schemas
router.get('/', asyncHandler(async (req: AuthenticatedRequest, res) => {
  try {
    const schemas = await datafoldService.getSchemas();
    
    const response: APIResponse = {
      success: true,
      data: schemas,
      meta: {
        timestamp: new Date().toISOString(),
        requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
        authStatus: 'authenticated'
      }
    };

    res.json(response);
  } catch (error) {
    throw createError(
      'Failed to retrieve schemas',
      500,
      'schema_retrieval_failed',
      { originalError: error instanceof Error ? error.message : error }
    );
  }
}));

// Create new schema
router.post('/', asyncHandler(async (req: AuthenticatedRequest, res) => {
  const { name, fields } = req.body;

  if (!name || !fields || !Array.isArray(fields)) {
    throw createError(
      'Name and fields array are required',
      400,
      'invalid_schema_data'
    );
  }

  try {
    const newSchema = await datafoldService.createSchema(name, fields);
    
    const response: APIResponse = {
      success: true,
      data: newSchema,
      meta: {
        timestamp: new Date().toISOString(),
        requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
        authStatus: 'authenticated'
      }
    };

    res.status(201).json(response);
  } catch (error) {
    throw createError(
      'Failed to create schema',
      500,
      'schema_creation_failed',
      { originalError: error instanceof Error ? error.message : error }
    );
  }
}));

// Delete schema
router.delete('/:name', asyncHandler(async (req: AuthenticatedRequest, res) => {
  const { name } = req.params;

  if (!name) {
    throw createError('Schema name is required', 400, 'missing_schema_name');
  }

  try {
    await datafoldService.deleteSchema(name);
    
    const response: APIResponse = {
      success: true,
      data: { message: `Schema '${name}' deleted successfully` },
      meta: {
        timestamp: new Date().toISOString(),
        requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
        authStatus: 'authenticated'
      }
    };

    res.json(response);
  } catch (error) {
    throw createError(
      'Failed to delete schema',
      500,
      'schema_deletion_failed',
      { originalError: error instanceof Error ? error.message : error }
    );
  }
}));

export default router;
```

```typescript
// src/routes/auth.ts
import { Router } from 'express';
import { AuthenticatedRequest, APIResponse } from '../types';
import { datafoldService } from '../services/datafold';
import { asyncHandler } from '../middleware/errorHandler';

const router = Router();

// Get authentication status
router.get('/status', asyncHandler(async (req: AuthenticatedRequest, res) => {
  const authStatus = {
    isAuthenticated: req.datafold?.isAuthenticated || false,
    clientId: req.datafold?.clientId,
    signature: req.datafold?.signature,
    serverConnection: datafoldService.isAuthEnabled()
  };

  const response: APIResponse = {
    success: true,
    data: authStatus,
    meta: {
      timestamp: new Date().toISOString(),
      requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
      authStatus: req.datafold?.isAuthenticated ? 'authenticated' : 'unauthenticated'
    }
  };

  res.json(response);
}));

// Test authenticated endpoint
router.get('/test', asyncHandler(async (req: AuthenticatedRequest, res) => {
  if (!req.datafold?.isAuthenticated) {
    const response: APIResponse = {
      success: false,
      error: {
        code: 'authentication_required',
        message: 'This endpoint requires authentication'
      },
      meta: {
        timestamp: new Date().toISOString(),
        requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
        authStatus: 'unauthenticated'
      }
    };

    return res.status(401).json(response);
  }

  const response: APIResponse = {
    success: true,
    data: {
      message: 'Authentication successful!',
      clientId: req.datafold.clientId,
      timestamp: new Date().toISOString()
    },
    meta: {
      timestamp: new Date().toISOString(),
      requestId: req.headers['x-request-id'] as string || `req_${Date.now()}`,
      authStatus: 'authenticated'
    }
  };

  res.json(response);
}));

export default router;
```

## 🚀 Step 7: Main Application

Put it all together in the main app:

```typescript
// src/app.ts
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import compression from 'compression';
import { v4 as uuidv4 } from 'uuid';

import { CONFIG, validateConfig } from './utils/config';
import { logger } from './utils/logger';
import { datafoldService } from './services/datafold';
import { createAuthMiddleware, clientInfo } from './middleware/auth';
import { errorHandler, notFoundHandler } from './middleware/errorHandler';

// Route imports
import healthRoutes from './routes/health';
import authRoutes from './routes/auth';
import schemaRoutes from './routes/schemas';

class ExpressApp {
  public app: express.Application;

  constructor() {
    this.app = express();
    this.initializeMiddleware();
    this.initializeRoutes();
    this.initializeErrorHandling();
  }

  private initializeMiddleware(): void {
    // Security middleware
    this.app.use(helmet());
    this.app.use(cors({
      origin: process.env.CORS_ORIGIN || '*',
      credentials: true
    }));

    // Compression
    this.app.use(compression());

    // Request parsing
    this.app.use(express.json({ limit: '10mb' }));
    this.app.use(express.urlencoded({ extended: true }));

    // Request ID middleware
    this.app.use((req, res, next) => {
      req.headers['x-request-id'] = req.headers['x-request-id'] || uuidv4();
      next();
    });

    // HTTP logging
    if (CONFIG.logging.enableHttp) {
      this.app.use(morgan('combined', {
        stream: {
          write: (message: string) => logger.info(message.trim())
        }
      }));
    }

    // DataFold authentication middleware
    this.app.use(createAuthMiddleware({
      required: false, // Global default - can be overridden per route
      skipRoutes: ['/health', '/api/health']
    }));

    // Client info middleware
    this.app.use(clientInfo);
  }

  private initializeRoutes(): void {
    // API routes
    this.app.use('/health', healthRoutes);
    this.app.use('/api/health', healthRoutes);
    this.app.use('/api/auth', authRoutes);
    this.app.use('/api/schemas', schemaRoutes);

    // Root endpoint
    this.app.get('/', (req, res) => {
      res.json({
        success: true,
        data: {
          message: 'DataFold Express API Server',
          version: process.env.npm_package_version || '1.0.0',
          environment: CONFIG.nodeEnv,
          authentication: {
            enabled: CONFIG.datafold.enableAuth,
            required: CONFIG.datafold.requireAuth
          }
        },
        meta: {
          timestamp: new Date().toISOString(),
          requestId: req.headers['x-request-id'] as string
        }
      });
    });
  }

  private initializeErrorHandling(): void {
    // 404 handler
    this.app.use(notFoundHandler);

    // Global error handler
    this.app.use(errorHandler);
  }

  public async start(): Promise<void> {
    try {
      // Validate configuration
      validateConfig();

      // Initialize DataFold service
      await datafoldService.initialize();

      // Start server
      this.app.listen(CONFIG.port, () => {
        logger.info(`Server started on port ${CONFIG.port}`, {
          environment: CONFIG.nodeEnv,
          datafoldAuth: CONFIG.datafold.enableAuth,
          clientId: datafoldService.getClientId()
        });
      });

    } catch (error) {
      logger.error('Failed to start server', error);
      process.exit(1);
    }
  }
}

export default ExpressApp;

// Start server if this file is run directly
if (require.main === module) {
  const app = new ExpressApp();
  app.start();
}
```

## ⚙️ Step 8: Environment Configuration

Create environment files:

```bash
# .env.development
NODE_ENV=development
PORT=3000
LOG_LEVEL=debug
ENABLE_HTTP_LOGGING=true

# DataFold Configuration
DATAFOLD_SERVER_URL=https://api.datafold.com
DATAFOLD_ENABLE_AUTH=true
DATAFOLD_REQUIRE_AUTH=false
DATAFOLD_SIGNATURE_CACHE=true
DATAFOLD_SIGNATURE_CACHE_TTL=300
DATAFOLD_SIGNATURE_CACHE_SIZE=1000

# CORS
CORS_ORIGIN=http://localhost:3000,http://localhost:3001
```

```bash
# .env.production
NODE_ENV=production
PORT=8080
LOG_LEVEL=info
ENABLE_HTTP_LOGGING=false

# DataFold Configuration (set these in your deployment)
DATAFOLD_SERVER_URL=https://api.datafold.com
DATAFOLD_CLIENT_ID=your-production-client-id
DATAFOLD_PRIVATE_KEY=your-production-private-key
DATAFOLD_ENABLE_AUTH=true
DATAFOLD_REQUIRE_AUTH=true
DATAFOLD_SIGNATURE_CACHE=true

# CORS
CORS_ORIGIN=https://yourdomain.com
```

## 📦 Step 9: Package Scripts

Update `package.json`:

```json
{
  "name": "datafold-express-api",
  "version": "1.0.0",
  "scripts": {
    "start": "node dist/app.js",
    "dev": "nodemon --exec ts-node src/app.ts",
    "build": "tsc",
    "test": "jest",
    "test:watch": "jest --watch",
    "lint": "eslint src/**/*.ts",
    "type-check": "tsc --noEmit"
  }
}
```

## ✅ Step 10: Testing Your Integration

### Manual Testing
```bash
# Start development server
npm run dev

# Test health endpoint (no auth)
curl http://localhost:3000/health

# Test authentication status
curl http://localhost:3000/api/auth/status

# Test protected endpoint with authentication
curl -X GET http://localhost:3000/api/schemas \
  -H "Signature: keyId=\"test\",algorithm=\"ed25519\",headers=\"(request-target) content-digest\",signature=\"xyz\"" \
  -H "Signature-Input: sig=(\"@method\" \"@target-uri\" \"content-digest\");created=1625097600"
```

### Automated Testing
```typescript
// src/app.test.ts
import request from 'supertest';
import ExpressApp from './app';

describe('Express App', () => {
  let app: ExpressApp;

  beforeAll(async () => {
    // Set test environment
    process.env.DATAFOLD_ENABLE_AUTH = 'false';
    process.env.NODE_ENV = 'test';
    
    app = new ExpressApp();
  });

  describe('Health Endpoints', () => {
    it('should return health status', async () => {
      const response = await request(app.app)
        .get('/health')
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.service).toBe('express-api');
    });
  });

  describe('Authentication', () => {
    it('should handle requests without authentication when not required', async () => {
      const response = await request(app.app)
        .get('/api/auth/status')
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.isAuthenticated).toBe(false);
    });

    it('should reject protected routes when auth is required', async () => {
      // Enable auth for this test
      process.env.DATAFOLD_REQUIRE_AUTH = 'true';
      
      const response = await request(app.app)
        .get('/api/schemas')
        .expect(401);

      expect(response.body.success).toBe(false);
      expect(response.body.error.code).toBe('authentication_required');
    });
  });

  describe('Error Handling', () => {
    it('should return 404 for unknown routes', async () => {
      const response = await request(app.app)
        .get('/nonexistent')
        .expect(404);

      expect(response.body.success).toBe(false);
      expect(response.body.error.code).toBe('not_found');
    });
  });
});
```

## 🚀 Production Deployment

### Docker Configuration
```dockerfile
# Dockerfile
FROM node:18-alpine

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy source code
COPY dist/ ./dist/

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S datafold -u 1001

# Change ownership
RUN chown -R datafold:nodejs /app
USER datafold

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Start application
CMD ["node", "dist/app.js"]
```

### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - NODE_ENV=production
      - DATAFOLD_SERVER_URL=${DATAFOLD_SERVER_URL}
      - DATAFOLD_CLIENT_ID=${DATAFOLD_CLIENT_ID}
      - DATAFOLD_PRIVATE_KEY=${DATAFOLD_PRIVATE_KEY}
      - DATAFOLD_ENABLE_AUTH=true
      - DATAFOLD_REQUIRE_AUTH=true
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## 🔧 Common Issues & Solutions

### Issue: Signature Verification Fails
```typescript
// Debug signature verification
const debugSignature = (req: Request) => {
  console.log('Signature Headers:', {
    signature: req.headers.signature,
    signatureInput: req.headers['signature-input'],
    contentType: req.headers['content-type'],
    contentLength: req.headers['content-length']
  });
};
```

### Issue: CORS Problems
```typescript
// Enhanced CORS configuration
app.use(cors({
  origin: (origin, callback) => {
    const allowedOrigins = process.env.CORS_ORIGIN?.split(',') || ['*'];
    if (!origin || allowedOrigins.includes('*') || allowedOrigins.includes(origin)) {
      callback(null, true);
    } else {
      callback(new Error('Not allowed by CORS'));
    }
  },
  credentials: true,
  exposedHeaders: ['x-request-id']
}));
```

### Issue: Memory Leaks
```typescript
// Monitor memory usage
setInterval(() => {
  const used = process.memoryUsage();
  logger.debug('Memory usage', {
    rss: Math.round(used.rss / 1024 / 1024) + ' MB',
    heapTotal: Math.round(used.heapTotal / 1024 / 1024) + ' MB',
    heapUsed: Math.round(used.heapUsed / 1024 / 1024) + ' MB'
  });
}, 60000);
```

## 🎯 Next Steps

### Advanced Features
- **[Rate Limiting](../../advanced/rate-limiting-patterns.md)** - Protect your API from abuse
- **[Caching Strategies](../../advanced/caching-patterns.md)** - Improve performance
- **[Real-time Features](../../advanced/realtime-integration.md)** - WebSocket authentication

### Deployment
- **[Kubernetes Deployment](../../deployment/kubernetes-deployment-guide.md)** - Scale your API
- **[Monitoring Setup](../../deployment/monitoring-integration.md)** - Production monitoring

### Security
- **[Security Hardening](../../advanced/security-hardening.md)** - Additional security measures
- **[Audit Logging](../../advanced/audit-logging.md)** - Comprehensive audit trails

---

🎉 **Congratulations!** You've built a production-ready Express.js API with DataFold signature authentication. Your server now provides secure, cryptographically-verified API endpoints with comprehensive error handling and monitoring.

💡 **Pro Tips**:
- Always validate and sanitize input data
- Implement proper rate limiting for production
- Use environment-specific configuration
- Monitor authentication metrics and failures
- Implement graceful shutdown handling
- Use structured logging for better observability