# Docker Integration Guide

Deploy DataFold signature authentication in containerized environments. This guide covers Docker best practices, secrets management, multi-stage builds, and production deployment patterns.

## 🎯 What You'll Build

A production-ready Docker setup featuring:
- 🐳 **Multi-stage Builds** - Optimized container images with security scanning
- 🔐 **Secure Secrets Management** - Private key handling without exposure
- 🚀 **Environment Configuration** - Flexible config for dev/staging/production
- 📊 **Health Monitoring** - Container health checks with authentication status
- 🔄 **Auto-scaling Ready** - Stateless authentication for horizontal scaling
- 🛡️ **Security Hardening** - Non-root users and minimal attack surface

## ⏱️ Estimated Time: 45 minutes

## 🛠️ Prerequisites

- Docker 20.10+ and Docker Compose
- Basic Docker knowledge (Dockerfile, containers, networking)
- Completed [Node.js/Express Tutorial](../frameworks/backend/nodejs-express-tutorial.md) or similar

## 🚀 Step 1: Application Setup

We'll containerize the Express.js API from the previous tutorial. If you haven't completed it, here's a minimal setup:

### Minimal Express App for Docker
```typescript
// src/app.ts
import express from 'express';
import { DataFoldClient, generateKeyPair } from '@datafold/sdk';

const app = express();
app.use(express.json());

let datafoldClient: DataFoldClient | null = null;

// Initialize DataFold authentication
async function initializeDataFold() {
  try {
    const serverUrl = process.env.DATAFOLD_SERVER_URL || 'https://api.datafold.com';
    const clientId = process.env.DATAFOLD_CLIENT_ID;
    const privateKey = process.env.DATAFOLD_PRIVATE_KEY;

    if (clientId && privateKey) {
      // Use existing credentials
      datafoldClient = new DataFoldClient({
        serverUrl,
        clientId,
        privateKey
      });
      console.log('✅ DataFold initialized with existing credentials');
    } else {
      // Generate new credentials
      const keyPair = await generateKeyPair();
      const newClientId = `docker-api-${Date.now()}`;

      // Register public key
      const response = await fetch(`${serverUrl}/api/crypto/keys/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          client_id: newClientId,
          public_key: keyPair.publicKey,
          key_name: 'Docker API Container',
          metadata: {
            container: true,
            environment: process.env.NODE_ENV || 'development'
          }
        })
      });

      const registration = await response.json();

      datafoldClient = new DataFoldClient({
        serverUrl,
        clientId: registration.data.client_id,
        privateKey: keyPair.privateKey
      });

      console.log('🔑 Generated new DataFold credentials:', {
        clientId: registration.data.client_id,
        privateKey: keyPair.privateKey
      });
    }

  } catch (error) {
    console.error('❌ DataFold initialization failed:', error);
    if (process.env.DATAFOLD_REQUIRED === 'true') {
      process.exit(1);
    }
  }
}

// Routes
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    datafold: datafoldClient ? 'authenticated' : 'not_authenticated'
  });
});

app.get('/api/test', async (req, res) => {
  if (!datafoldClient) {
    return res.status(503).json({ error: 'DataFold not available' });
  }

  try {
    const response = await datafoldClient.get('/api/system/status');
    res.json({ success: true, data: response.data });
  } catch (error) {
    res.status(500).json({ error: 'DataFold request failed' });
  }
});

const PORT = process.env.PORT || 3000;

// Initialize and start server
initializeDataFold().then(() => {
  app.listen(PORT, () => {
    console.log(`🚀 Server running on port ${PORT}`);
  });
});
```

## 🐳 Step 2: Basic Dockerfile

Start with a basic Dockerfile:

```dockerfile
# Dockerfile.basic
FROM node:18-alpine

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy application code
COPY dist/ ./dist/

# Expose port
EXPOSE 3000

# Start application
CMD ["node", "dist/app.js"]
```

### Build and Test Basic Image
```bash
# Build TypeScript
npm run build

# Build Docker image
docker build -f Dockerfile.basic -t datafold-api:basic .

# Run container
docker run -p 3000:3000 \
  -e DATAFOLD_SERVER_URL=https://api.datafold.com \
  datafold-api:basic

# Test
curl http://localhost:3000/health
```

## 🏗️ Step 3: Multi-Stage Production Dockerfile

Create an optimized production Dockerfile:

```dockerfile
# Dockerfile
# Multi-stage build for optimal image size and security

#
# Stage 1: Build Environment
#
FROM node:18-alpine AS builder

# Install build dependencies
RUN apk add --no-cache python3 make g++

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./

# Install all dependencies (including dev)
RUN npm ci

# Copy source code
COPY src/ ./src/

# Build application
RUN npm run build

# Remove dev dependencies
RUN npm prune --production

#
# Stage 2: Runtime Environment
#
FROM node:18-alpine AS runtime

# Create app user (security best practice)
RUN addgroup -g 1001 -S datafold && \
    adduser -D -S -G datafold -u 1001 datafold

# Install runtime dependencies only
RUN apk add --no-cache \
    ca-certificates \
    curl \
    dumb-init && \
    rm -rf /var/cache/apk/*

# Set working directory
WORKDIR /app

# Copy package files and node_modules from builder
COPY --from=builder --chown=datafold:datafold /app/package*.json ./
COPY --from=builder --chown=datafold:datafold /app/node_modules ./node_modules/

# Copy built application
COPY --from=builder --chown=datafold:datafold /app/dist ./dist/

# Create logs directory
RUN mkdir -p /app/logs && chown datafold:datafold /app/logs

# Switch to non-root user
USER datafold

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Use dumb-init for proper signal handling
ENTRYPOINT ["dumb-init", "--"]

# Start application
CMD ["node", "dist/app.js"]
```

### Build Optimized Image
```bash
# Build optimized image
docker build -t datafold-api:latest .

# Check image size
docker images datafold-api

# Run with health check
docker run -p 3000:3000 \
  --name datafold-api \
  -e DATAFOLD_SERVER_URL=https://api.datafold.com \
  datafold-api:latest
```

## 🔐 Step 4: Secure Secrets Management

### Option 1: Docker Secrets (Swarm/Compose)

```yaml
# docker-compose.secrets.yml
version: '3.8'

services:
  api:
    image: datafold-api:latest
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DATAFOLD_SERVER_URL=https://api.datafold.com
      - DATAFOLD_CLIENT_ID_FILE=/run/secrets/datafold_client_id
      - DATAFOLD_PRIVATE_KEY_FILE=/run/secrets/datafold_private_key
    secrets:
      - datafold_client_id
      - datafold_private_key
    deploy:
      replicas: 2
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3

secrets:
  datafold_client_id:
    file: ./secrets/client_id.txt
  datafold_private_key:
    file: ./secrets/private_key.txt
```

Update your app to read from secret files:
```typescript
// Enhanced app.ts for secrets
function readSecret(filePath: string): string | null {
  try {
    const fs = require('fs');
    return fs.readFileSync(filePath, 'utf8').trim();
  } catch (error) {
    console.warn(`Could not read secret from ${filePath}:`, error.message);
    return null;
  }
}

async function initializeDataFold() {
  const serverUrl = process.env.DATAFOLD_SERVER_URL || 'https://api.datafold.com';
  
  // Try to read from secret files first, then environment variables
  const clientId = 
    readSecret(process.env.DATAFOLD_CLIENT_ID_FILE || '') ||
    process.env.DATAFOLD_CLIENT_ID;
    
  const privateKey = 
    readSecret(process.env.DATAFOLD_PRIVATE_KEY_FILE || '') ||
    process.env.DATAFOLD_PRIVATE_KEY;

  // ... rest of initialization
}
```

### Option 2: Init Container Pattern

```yaml
# docker-compose.init.yml
version: '3.8'

services:
  # Init container to generate/retrieve credentials
  datafold-init:
    image: datafold-api:latest
    command: ["node", "-e", "
      const { generateKeyPair } = require('@datafold/sdk');
      const fs = require('fs');
      
      (async () => {
        const keyPair = await generateKeyPair();
        const clientId = `docker-init-${Date.now()}`;
        
        // Register with DataFold
        const response = await fetch('${DATAFOLD_SERVER_URL}/api/crypto/keys/register', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            client_id: clientId,
            public_key: keyPair.publicKey,
            key_name: 'Docker Init Container'
          })
        });
        
        const registration = await response.json();
        
        // Write credentials to shared volume
        fs.writeFileSync('/shared/client_id', registration.data.client_id);
        fs.writeFileSync('/shared/private_key', keyPair.privateKey);
        
        console.log('Credentials generated and saved');
      })();
    "]
    environment:
      - DATAFOLD_SERVER_URL=https://api.datafold.com
    volumes:
      - credentials:/shared

  # Main application container
  api:
    image: datafold-api:latest
    depends_on:
      - datafold-init
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DATAFOLD_SERVER_URL=https://api.datafold.com
      - DATAFOLD_CLIENT_ID_FILE=/shared/client_id
      - DATAFOLD_PRIVATE_KEY_FILE=/shared/private_key
    volumes:
      - credentials:/shared:ro

volumes:
  credentials:
```

### Option 3: External Secret Management

```yaml
# docker-compose.vault.yml
version: '3.8'

services:
  # Vault agent sidecar
  vault-agent:
    image: vault:latest
    command: vault agent -config=/vault/config/agent.hcl
    volumes:
      - vault-config:/vault/config
      - vault-secrets:/vault/secrets
    environment:
      - VAULT_ADDR=https://vault.yourdomain.com

  # Main application
  api:
    image: datafold-api:latest
    depends_on:
      - vault-agent
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DATAFOLD_SERVER_URL=https://api.datafold.com
      - DATAFOLD_CLIENT_ID_FILE=/vault/secrets/client_id
      - DATAFOLD_PRIVATE_KEY_FILE=/vault/secrets/private_key
    volumes:
      - vault-secrets:/vault/secrets:ro

volumes:
  vault-config:
  vault-secrets:
```

## 🌍 Step 5: Environment-Specific Configuration

### Development Environment
```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  api:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=development
      - LOG_LEVEL=debug
      - DATAFOLD_SERVER_URL=http://host.docker.internal:8080
      - DATAFOLD_REQUIRED=false
    volumes:
      - ./src:/app/src
      - ./logs:/app/logs
    command: ["npm", "run", "dev"]

# Development Dockerfile
# Dockerfile.dev
FROM node:18-alpine

RUN apk add --no-cache curl

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

EXPOSE 3000

CMD ["npm", "run", "dev"]
```

### Staging Environment
```yaml
# docker-compose.staging.yml
version: '3.8'

services:
  api:
    image: datafold-api:latest
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=staging
      - LOG_LEVEL=info
      - DATAFOLD_SERVER_URL=https://staging-api.datafold.com
      - DATAFOLD_CLIENT_ID=${DATAFOLD_STAGING_CLIENT_ID}
      - DATAFOLD_PRIVATE_KEY=${DATAFOLD_STAGING_PRIVATE_KEY}
      - DATAFOLD_REQUIRED=true
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```

### Production Environment
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  api:
    image: datafold-api:latest
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - LOG_LEVEL=info
      - DATAFOLD_SERVER_URL=https://api.datafold.com
      - DATAFOLD_REQUIRED=true
    secrets:
      - datafold_client_id
      - datafold_private_key
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
        order: start-first
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
        reservations:
          memory: 512M
          cpus: '0.25'
    networks:
      - app-network
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  # Nginx reverse proxy
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - api
    networks:
      - app-network

secrets:
  datafold_client_id:
    external: true
  datafold_private_key:
    external: true

networks:
  app-network:
    driver: overlay
```

## 📊 Step 6: Advanced Health Monitoring

### Enhanced Health Check
```typescript
// src/health.ts
import { DataFoldClient } from '@datafold/sdk';

export interface HealthStatus {
  status: 'healthy' | 'unhealthy' | 'degraded';
  timestamp: string;
  checks: {
    [key: string]: {
      status: 'pass' | 'fail' | 'warn';
      time: string;
      output?: string;
    };
  };
}

export async function performHealthCheck(datafoldClient: DataFoldClient | null): Promise<HealthStatus> {
  const checks: HealthStatus['checks'] = {};
  let overallStatus: 'healthy' | 'unhealthy' | 'degraded' = 'healthy';

  // Check DataFold connectivity
  try {
    const start = Date.now();
    if (datafoldClient) {
      await datafoldClient.get('/api/system/status');
      checks.datafold = {
        status: 'pass',
        time: `${Date.now() - start}ms`,
        output: 'DataFold API accessible'
      };
    } else {
      checks.datafold = {
        status: 'warn',
        time: '0ms',
        output: 'DataFold client not initialized'
      };
      overallStatus = 'degraded';
    }
  } catch (error) {
    checks.datafold = {
      status: 'fail',
      time: '0ms',
      output: `DataFold connectivity failed: ${error.message}`
    };
    overallStatus = 'unhealthy';
  }

  // Check memory usage
  const memUsage = process.memoryUsage();
  const memUsageMB = Math.round(memUsage.heapUsed / 1024 / 1024);
  checks.memory = {
    status: memUsageMB < 200 ? 'pass' : memUsageMB < 400 ? 'warn' : 'fail',
    time: '0ms',
    output: `${memUsageMB}MB heap used`
  };

  if (checks.memory.status === 'fail') {
    overallStatus = 'unhealthy';
  } else if (checks.memory.status === 'warn' && overallStatus === 'healthy') {
    overallStatus = 'degraded';
  }

  // Check disk space (if applicable)
  try {
    const fs = require('fs');
    const stats = fs.statSync('/app');
    checks.disk = {
      status: 'pass',
      time: '0ms',
      output: 'Disk accessible'
    };
  } catch (error) {
    checks.disk = {
      status: 'fail',
      time: '0ms',
      output: 'Disk check failed'
    };
    overallStatus = 'unhealthy';
  }

  return {
    status: overallStatus,
    timestamp: new Date().toISOString(),
    checks
  };
}

// Enhanced health endpoint
app.get('/health', async (req, res) => {
  const health = await performHealthCheck(datafoldClient);
  const statusCode = health.status === 'healthy' ? 200 : 
                    health.status === 'degraded' ? 200 : 503;
  
  res.status(statusCode).json(health);
});

// Detailed health endpoint
app.get('/health/detailed', async (req, res) => {
  const health = await performHealthCheck(datafoldClient);
  
  const detailed = {
    ...health,
    system: {
      uptime: process.uptime(),
      version: process.version,
      platform: process.platform,
      memory: process.memoryUsage(),
      cpuUsage: process.cpuUsage()
    },
    environment: {
      nodeEnv: process.env.NODE_ENV,
      port: process.env.PORT,
      datafoldEnabled: !!datafoldClient
    }
  };

  res.json(detailed);
});
```

### Docker Health Check Script
```bash
#!/bin/sh
# healthcheck.sh

# Set timeout
TIMEOUT=10

# Check main health endpoint
if ! curl -f --max-time $TIMEOUT http://localhost:3000/health >/dev/null 2>&1; then
  echo "Health check failed"
  exit 1
fi

# Check if DataFold authentication is working (optional)
if [ "$DATAFOLD_REQUIRED" = "true" ]; then
  if ! curl -f --max-time $TIMEOUT http://localhost:3000/api/test >/dev/null 2>&1; then
    echo "DataFold authentication check failed"
    exit 1
  fi
fi

echo "Health check passed"
exit 0
```

Update Dockerfile:
```dockerfile
# Copy health check script
COPY healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/healthcheck.sh

# Enhanced health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD ["/usr/local/bin/healthcheck.sh"]
```

## 🔄 Step 7: Container Orchestration

### Docker Swarm Deployment
```bash
# Initialize swarm
docker swarm init

# Create secrets
echo "your-client-id" | docker secret create datafold_client_id -
echo "your-private-key" | docker secret create datafold_private_key -

# Deploy stack
docker stack deploy -c docker-compose.prod.yml datafold-api

# Check services
docker service ls
docker service logs datafold-api_api
```

### Kubernetes Deployment
```yaml
# k8s-deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: datafold-api
  labels:
    app: datafold-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: datafold-api
  template:
    metadata:
      labels:
        app: datafold-api
    spec:
      containers:
      - name: api
        image: datafold-api:latest
        ports:
        - containerPort: 3000
        env:
        - name: NODE_ENV
          value: "production"
        - name: DATAFOLD_SERVER_URL
          value: "https://api.datafold.com"
        - name: DATAFOLD_CLIENT_ID
          valueFrom:
            secretKeyRef:
              name: datafold-secrets
              key: client-id
        - name: DATAFOLD_PRIVATE_KEY
          valueFrom:
            secretKeyRef:
              name: datafold-secrets
              key: private-key
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"

---
apiVersion: v1
kind: Service
metadata:
  name: datafold-api-service
spec:
  selector:
    app: datafold-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer

---
apiVersion: v1
kind: Secret
metadata:
  name: datafold-secrets
type: Opaque
data:
  client-id: <base64-encoded-client-id>
  private-key: <base64-encoded-private-key>
```

## 🛡️ Step 8: Security Hardening

### Security-Enhanced Dockerfile
```dockerfile
# Dockerfile.secure
FROM node:18-alpine AS builder

# Install security updates
RUN apk update && apk upgrade && apk add --no-cache \
    python3 make g++ && \
    rm -rf /var/cache/apk/*

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build && npm prune --production

# Security-hardened runtime
FROM gcr.io/distroless/nodejs18-debian11 AS runtime

# Copy only necessary files
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# Run as non-root user
USER 1001

EXPOSE 3000

CMD ["dist/app.js"]
```

### Security Scanning
```bash
# Scan for vulnerabilities
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  -v $HOME/Library/Caches:/root/.cache/ \
  aquasec/trivy image datafold-api:latest

# Scan Node.js dependencies
npm audit
npm audit fix

# Use hadolint for Dockerfile linting
docker run --rm -i hadolint/hadolint < Dockerfile
```

### Runtime Security
```yaml
# docker-compose.secure.yml
version: '3.8'

services:
  api:
    image: datafold-api:secure
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    read_only: true
    tmpfs:
      - /tmp
      - /app/logs
    security_opt:
      - no-new-privileges:true
    user: "1001:1001"
    environment:
      - NODE_ENV=production
    networks:
      - app-network

networks:
  app-network:
    driver: bridge
    internal: true
```

## 🔧 Step 9: Performance Optimization

### Multi-Process Setup
```typescript
// src/cluster.ts
import cluster from 'cluster';
import os from 'os';

if (cluster.isPrimary) {
  const numCPUs = os.cpus().length;
  console.log(`Primary ${process.pid} is running`);

  // Fork workers
  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }

  cluster.on('exit', (worker, code, signal) => {
    console.log(`Worker ${worker.process.pid} died`);
    cluster.fork();
  });
} else {
  // Worker process
  require('./app');
  console.log(`Worker ${process.pid} started`);
}
```

### Resource Optimization
```dockerfile
# Use alpine for smaller size
FROM node:18-alpine

# Optimize npm install
RUN npm config set registry https://registry.npmjs.org/ && \
    npm config set cache /tmp/.npm

# Remove unnecessary files
RUN rm -rf /usr/share/man/* && \
    rm -rf /tmp/* && \
    rm -rf /var/cache/apk/*

# Use .dockerignore
# .dockerignore
node_modules
npm-debug.log
.git
.gitignore
README.md
.env
.nyc_output
coverage
.nyc_output
```

## ✅ Step 10: Testing and Validation

### Integration Testing
```bash
#!/bin/bash
# test-docker-integration.sh

set -e

echo "🧪 Testing Docker integration..."

# Build image
docker build -t datafold-api:test .

# Start container
docker run -d --name test-api \
  -p 3001:3000 \
  -e DATAFOLD_SERVER_URL=https://api.datafold.com \
  -e NODE_ENV=test \
  datafold-api:test

# Wait for startup
sleep 10

# Test health endpoint
echo "Testing health endpoint..."
if curl -f http://localhost:3001/health; then
  echo "✅ Health check passed"
else
  echo "❌ Health check failed"
  exit 1
fi

# Test authenticated endpoint
echo "Testing authenticated endpoint..."
if curl -f http://localhost:3001/api/test; then
  echo "✅ Authentication test passed"
else
  echo "⚠️ Authentication test failed (may be expected)"
fi

# Check logs
echo "📋 Container logs:"
docker logs test-api

# Cleanup
docker stop test-api
docker rm test-api

echo "🎉 Docker integration tests completed!"
```

### Load Testing
```yaml
# docker-compose.loadtest.yml
version: '3.8'

services:
  api:
    image: datafold-api:latest
    deploy:
      replicas: 3

  loadtest:
    image: loadimpact/k6:latest
    volumes:
      - ./loadtest.js:/loadtest.js
    command: run --vus 50 --duration 2m /loadtest.js
    depends_on:
      - api
```

```javascript
// loadtest.js
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 50,
  duration: '2m',
};

export default function() {
  let response = http.get('http://api:3000/health');
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

## 🚀 Production Best Practices

### 1. Logging Configuration
```typescript
// Enhanced logging for containers
import winston from 'winston';

const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: {
    service: 'datafold-api',
    container: process.env.HOSTNAME,
    version: process.env.npm_package_version
  },
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ 
      filename: '/app/logs/error.log', 
      level: 'error' 
    }),
    new winston.transports.File({ 
      filename: '/app/logs/combined.log' 
    })
  ]
});
```

### 2. Graceful Shutdown
```typescript
// Graceful shutdown handling
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully');
  server.close(() => {
    console.log('Process terminated');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  console.log('SIGINT received, shutting down gracefully');
  server.close(() => {
    console.log('Process terminated');
    process.exit(0);
  });
});
```

### 3. Monitoring Integration
```yaml
# Prometheus monitoring
services:
  api:
    image: datafold-api:latest
    labels:
      - "prometheus.scrape=true"
      - "prometheus.port=3000"
      - "prometheus.path=/metrics"

  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
```

## 🔧 Common Issues & Solutions

### Issue: Container Won't Start
```bash
# Debug container startup
docker run -it --entrypoint /bin/sh datafold-api:latest

# Check logs
docker logs --follow container-name

# Check resource constraints
docker stats container-name
```

### Issue: Authentication Fails in Container
```bash
# Test network connectivity
docker exec -it container-name curl -v https://api.datafold.com

# Check environment variables
docker exec -it container-name env | grep DATAFOLD

# Verify secrets mounting
docker exec -it container-name ls -la /run/secrets/
```

### Issue: Health Checks Failing
```bash
# Debug health check
docker exec -it container-name curl -v http://localhost:3000/health

# Check health check script
docker exec -it container-name /usr/local/bin/healthcheck.sh
```

## 🎯 Next Steps

### Advanced Container Patterns
- **[Kubernetes Integration](kubernetes-deployment-guide.md)** - Full K8s deployment
- **[Service Mesh](../advanced/service-mesh-integration.md)** - Istio/Linkerd integration
- **[Observability](../advanced/observability-patterns.md)** - Metrics, tracing, logging

### CI/CD Integration
- **[CI/CD Pipeline](ci-cd-integration-tutorial.md)** - Automated Docker builds
- **[Registry Management](../advanced/container-registry.md)** - Private registries
- **[Security Scanning](../advanced/security-scanning.md)** - Automated vulnerability scanning

---

🎉 **Congratulations!** You've successfully containerized your DataFold-authenticated application with production-ready Docker configurations. Your containers are now secure, scalable, and ready for deployment in any environment.

💡 **Pro Tips**:
- Always use multi-stage builds for production
- Never embed secrets in Docker images
- Implement proper health checks for orchestration
- Use non-root users for security
- Monitor container resource usage
- Implement graceful shutdown handling
- Use security scanning in your CI/CD pipeline