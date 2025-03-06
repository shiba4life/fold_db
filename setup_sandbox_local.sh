#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up Datafold Sandbox Environment${NC}"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Docker Compose is not installed. Please install Docker Compose first.${NC}"
    exit 1
fi

# Create the internal Docker network if it doesn't exist
echo -e "${YELLOW}Creating internal Docker network...${NC}"
NETWORK_EXISTS=$(docker network ls --filter name=datafold_internal_network --format "{{.Name}}")
if [ -z "$NETWORK_EXISTS" ]; then
    docker network create --internal datafold_internal_network
    echo -e "${GREEN}Network created successfully.${NC}"
else
    echo -e "${YELLOW}Network already exists.${NC}"
fi

# Create the Unix socket directory locally
echo -e "${YELLOW}Creating local Unix socket directory...${NC}"
mkdir -p ./socket
chmod 777 ./socket

# Modify docker-compose.yml to use local socket directory
echo -e "${YELLOW}Creating modified docker-compose.yml...${NC}"
cat > docker-compose-local.yml << EOF
version: '3.8'

networks:
  datafold_internal_network:
    driver: bridge
    internal: true

services:
  datafold-api:
    build:
      context: .
      dockerfile: Dockerfile.local
    container_name: datafold-api
    networks:
      - datafold_internal_network
    ports:
      - "8080:8080"  # Optional: expose API port externally
    volumes:
      - ./data:/data
      - ./socket:/var/run
    environment:
      - RUST_LOG=info
      - USE_UNIX_SOCKET=true
      - UNIX_SOCKET_PATH=/var/run/datafold.sock
      - API_PORT=8080
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE

  # Example of a sandboxed third-party container
  # This is just a template and would be created dynamically by the SandboxManager
  sandboxed-app:
    image: \${APP_IMAGE:-alpine}
    container_name: sandboxed-app
    networks:
      - datafold_internal_network
    security_opt:
      - no-new-privileges:true
      - seccomp=default
    cap_drop:
      - ALL
    read_only: true
    tmpfs:
      - /tmp
    environment:
      - DATAFOLD_API_HOST=datafold-api
      - DATAFOLD_API_PORT=8080
    mem_limit: 512m
    cpus: 0.5
    pids_limit: 100
    depends_on:
      - datafold-api
EOF

# Build the Datafold API container
echo -e "${YELLOW}Building Datafold API container...${NC}"
docker-compose -f docker-compose-local.yml build datafold-api

# Start the Datafold API container
echo -e "${YELLOW}Starting Datafold API container...${NC}"
docker-compose -f docker-compose-local.yml up -d datafold-api

# Wait for the API container to be ready
echo -e "${YELLOW}Waiting for Datafold API to be ready...${NC}"
MAX_RETRIES=30
RETRY_COUNT=0
API_READY=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if docker logs datafold-api 2>&1 | grep -q "Datafold API ready"; then
        API_READY=true
        break
    fi
    echo -e "${YELLOW}Waiting for API to start (${RETRY_COUNT}/${MAX_RETRIES})...${NC}"
    RETRY_COUNT=$((RETRY_COUNT+1))
    sleep 2
done

if [ "$API_READY" = true ]; then
    echo -e "${GREEN}Datafold API is ready!${NC}"
else
    echo -e "${YELLOW}Datafold API may not be fully initialized yet, but continuing...${NC}"
fi

echo -e "${GREEN}Datafold Sandbox Environment is ready!${NC}"
echo ""
echo -e "${YELLOW}To run a sandboxed container, use:${NC}"
echo "docker run --rm \\"
echo "  --network=datafold_internal_network \\"
echo "  --cap-drop=ALL \\"
echo "  --security-opt no-new-privileges \\"
echo "  --env DATAFOLD_API_HOST=datafold-api \\"
echo "  --env DATAFOLD_API_PORT=8080 \\"
echo "  your-image-name"
echo ""
echo -e "${YELLOW}For Unix socket communication, use:${NC}"
echo "docker run --rm \\"
echo "  --network=none \\"
echo "  --cap-drop=ALL \\"
echo "  --security-opt no-new-privileges \\"
echo "  -v ./socket/datafold.sock:/datafold.sock \\"
echo "  --env DATAFOLD_API_SOCKET=/datafold.sock \\"
echo "  your-image-name"
echo ""
echo -e "${YELLOW}To stop the environment:${NC}"
echo "docker-compose -f docker-compose-local.yml down"
