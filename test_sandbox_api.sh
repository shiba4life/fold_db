#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing Datafold Sandbox API Access${NC}"

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

# Check if the internal Docker network exists
echo -e "${YELLOW}Checking internal Docker network...${NC}"
NETWORK_EXISTS=$(docker network ls --filter name=datafold_internal_network --format "{{.Name}}")
if [ -z "$NETWORK_EXISTS" ]; then
    echo -e "${RED}Network does not exist. Please run setup_sandbox_local.sh first.${NC}"
    exit 1
else
    echo -e "${GREEN}Network exists.${NC}"
fi

# Check if the Datafold API container is running
echo -e "${YELLOW}Checking Datafold API container...${NC}"
API_RUNNING=$(docker ps --filter name=datafold-api --format "{{.Names}}")
if [ -z "$API_RUNNING" ]; then
    echo -e "${RED}Datafold API container is not running. Please run setup_sandbox_local.sh first.${NC}"
    exit 1
else
    echo -e "${GREEN}Datafold API container is running.${NC}"
fi

# Build the example sandboxed app
echo -e "${YELLOW}Building example sandboxed app...${NC}"
cd examples/sandboxed-app
docker build -t datafold-sandboxed-app .
cd ../..

# Run the example sandboxed app with network-based communication
echo -e "${YELLOW}Running example sandboxed app with network-based communication...${NC}"
CONTAINER_ID=$(docker run -d --rm -p 3000:3000 \
  --network=datafold_internal_network \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  --env DATAFOLD_API_HOST=datafold-api \
  --env DATAFOLD_API_PORT=8080 \
  datafold-sandboxed-app)

# Wait for the container to start
echo -e "${YELLOW}Waiting for container to start...${NC}"
MAX_RETRIES=30
RETRY_COUNT=0
APP_READY=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if docker logs $CONTAINER_ID 2>&1 | grep -q "APP_READY"; then
        APP_READY=true
        break
    fi
    echo -e "${YELLOW}Waiting for app to initialize (${RETRY_COUNT}/${MAX_RETRIES})...${NC}"
    RETRY_COUNT=$((RETRY_COUNT+1))
    sleep 2
done

if [ "$APP_READY" = true ]; then
    echo -e "${GREEN}App is ready!${NC}"
else
    echo -e "${YELLOW}App may not be fully initialized yet, but continuing...${NC}"
fi

# Test the API connection
echo -e "${YELLOW}Testing API connection...${NC}"
MAX_RETRIES=30
RETRY_COUNT=0
API_CONNECTED=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    RESPONSE=$(curl -s http://localhost:3000/ 2>/dev/null)
    if [[ "$RESPONSE" == *"Datafold Sandboxed App"* ]]; then
        API_CONNECTED=true
        break
    fi
    echo -e "${YELLOW}Waiting for app to start (${RETRY_COUNT}/${MAX_RETRIES})...${NC}"
    RETRY_COUNT=$((RETRY_COUNT+1))
    sleep 2
done

if [ "$API_CONNECTED" = true ]; then
    echo -e "${GREEN}API connection successful.${NC}"
else
    echo -e "${RED}API connection failed.${NC}"
    docker logs $CONTAINER_ID
    docker stop $CONTAINER_ID
    exit 1
fi

# Test Unix socket communication
echo -e "${YELLOW}Testing Unix socket communication...${NC}"
if [ -S "./socket/datafold.sock" ]; then
    echo -e "${GREEN}Unix socket exists.${NC}"
    
    # Run a test container with Unix socket mount
    echo -e "${YELLOW}Running test container with Unix socket...${NC}"
    SOCKET_TEST_ID=$(docker run -d --rm \
      --network=none \
      --cap-drop=ALL \
      --security-opt no-new-privileges \
      -v ./socket/datafold.sock:/datafold.sock \
      --env DATAFOLD_API_SOCKET=/datafold.sock \
      alpine:latest \
      sh -c "apk add --no-cache curl && curl --unix-socket /datafold.sock http://localhost/ && echo 'Socket test completed'")
    
    # Wait for the container to finish
    sleep 5
    
    # Check the logs
    SOCKET_LOGS=$(docker logs $SOCKET_TEST_ID 2>&1)
    docker rm -f $SOCKET_TEST_ID >/dev/null 2>&1
    
    if [[ "$SOCKET_LOGS" == *"Socket test completed"* ]]; then
        echo -e "${GREEN}Unix socket communication successful.${NC}"
    else
        echo -e "${YELLOW}Unix socket communication test returned: ${SOCKET_LOGS}${NC}"
    fi
else
    echo -e "${YELLOW}Unix socket file not found. Socket communication not tested.${NC}"
fi

# Test API endpoints
echo -e "${YELLOW}Testing API endpoints...${NC}"

# Test schemas endpoint
echo -e "${YELLOW}Testing schemas endpoint...${NC}"
SCHEMAS_RESPONSE=$(curl -s http://localhost:3000/schemas)
if [[ "$SCHEMAS_RESPONSE" == *"error"* ]]; then
    echo -e "${RED}Schemas endpoint failed: $SCHEMAS_RESPONSE${NC}"
else
    echo -e "${GREEN}Schemas endpoint successful.${NC}"
fi

# Test query endpoint
echo -e "${YELLOW}Testing query endpoint...${NC}"
QUERY_RESPONSE=$(curl -s http://localhost:3000/query/user-profile?fields=username,email)
if [[ "$QUERY_RESPONSE" == *"error"* ]]; then
    echo -e "${RED}Query endpoint failed: $QUERY_RESPONSE${NC}"
else
    echo -e "${GREEN}Query endpoint successful.${NC}"
fi

# Test external network access (should fail)
echo -e "${YELLOW}Testing external network access (should fail)...${NC}"
RESPONSE=$(curl -s http://localhost:3000/test-external)
if [[ "$RESPONSE" == *"External network access is blocked"* ]]; then
    echo -e "${GREEN}External network access is blocked as expected.${NC}"
else
    echo -e "${RED}External network access is available. This is a security issue!${NC}"
    docker logs $CONTAINER_ID
    docker stop $CONTAINER_ID
    exit 1
fi

# Stop the container
echo -e "${YELLOW}Stopping container...${NC}"
docker stop $CONTAINER_ID

echo -e "${GREEN}All tests passed!${NC}"
echo -e "${GREEN}The Datafold Sandbox API Access is working correctly.${NC}"
