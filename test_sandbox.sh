#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing Datafold Sandbox Environment${NC}"

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
    echo -e "${RED}Network does not exist. Please run setup_sandbox.sh first.${NC}"
    exit 1
else
    echo -e "${GREEN}Network exists.${NC}"
fi

# Check if the Datafold API container is running
echo -e "${YELLOW}Checking Datafold API container...${NC}"
API_RUNNING=$(docker ps --filter name=datafold-api --format "{{.Names}}")
if [ -z "$API_RUNNING" ]; then
    echo -e "${RED}Datafold API container is not running. Please run setup_sandbox.sh first.${NC}"
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
echo -e "${GREEN}The Datafold Sandbox Environment is working correctly.${NC}"
