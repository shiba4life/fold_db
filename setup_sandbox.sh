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

# Create the Unix socket directory if it doesn't exist
echo -e "${YELLOW}Creating Unix socket directory...${NC}"
mkdir -p /var/run
chmod 777 /var/run

# Build the Datafold API container
echo -e "${YELLOW}Building Datafold API container...${NC}"
docker-compose build datafold-api

# Start the Datafold API container
echo -e "${YELLOW}Starting Datafold API container...${NC}"
docker-compose up -d datafold-api

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
echo "  -v /var/run/datafold.sock:/datafold.sock \\"
echo "  --env DATAFOLD_API_SOCKET=/datafold.sock \\"
echo "  your-image-name"
echo ""
echo -e "${YELLOW}To stop the environment:${NC}"
echo "docker-compose down"
