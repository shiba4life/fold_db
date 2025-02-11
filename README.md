# DataFold Node - Container Manager

DataFold Node is a containerized service that manages and runs Docker applications. It provides a simple API for deploying and managing containerized applications while controlling their network access and resource usage.

## Overview

DataFold Node runs as a container that:
- Uses the host's Docker daemon to manage other containers
- Provides REST API for container management
- Controls network access for managed containers
- Monitors container health and resource usage

## Quick Start

1. Prerequisites:
   - Docker 20.10.0+
   - Docker socket access (/var/run/docker.sock)

2. Start DataFold Node:
   ```bash
   docker-compose up -d
   ```

3. Deploy an application:
   ```bash
   curl -X POST http://localhost:8080/api/apps/deploy \
     -H "Content-Type: application/json" \
     -d '{
       "image": "nginx:latest",
       "name": "web-app",
       "network_access": "restricted",
       "ports": {"80": "8081"}
     }'
   ```

## API Reference

### Application Management

1. Deploy Application
```bash
POST /api/apps/deploy
{
  "image": "image:tag",
  "name": "app-name",
  "network_access": "restricted|internal|external",
  "ports": {"container_port": "host_port"},
  "resources": {
    "cpu_limit": "0.5",
    "memory_limit": "512m"
  }
}
```

2. Control Applications
```bash
# Start
POST /api/apps/start/{name}

# Stop
POST /api/apps/stop/{name}

# Remove
DELETE /api/apps/remove/{name}
```

3. Monitor Applications
```bash
# List all apps
GET /api/apps/list

# Get app status
GET /api/apps/status/{name}

# Get app logs
GET /api/apps/logs/{name}
```

## Network Access Levels

- `restricted`: Only allowed ports exposed
- `internal`: Communication between containers only
- `external`: Full network access

## Configuration

### docker-compose.yml
```yaml
services:
  datafold_node:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
```

### Environment Variables
- `NODE_CONFIG`: Path to config file
- `DOCKER_HOST`: Docker socket path

## Security

- Applications run with restricted network access by default
- Docker socket mounted read-only
- Resource limits enforced for all containers
- API authentication required

## Monitoring

- Container health checks
- Resource usage monitoring
- Application logs access
- Status reporting

## Best Practices

1. Always specify:
   - Network access level
   - Resource limits
   - Required ports only

2. Use:
   - Specific image tags
   - Health checks
   - Resource constraints

## Troubleshooting

1. Check logs:
   ```bash
   docker-compose logs datafold_node
   ```

2. Verify permissions:
   ```bash
   ls -l /var/run/docker.sock
   ```

3. Monitor containers:
   ```bash
   docker ps
   docker stats
   ```
