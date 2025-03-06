#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running Datafold Sandbox API Demo${NC}"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo -e "${YELLOW}Docker Compose is not installed. Attempting to use Docker Compose plugin...${NC}"
    # Try using the Docker Compose plugin (docker compose) instead
    if ! command -v docker &> /dev/null || ! docker compose version &> /dev/null; then
        echo -e "${RED}Neither docker-compose nor Docker Compose plugin is available.${NC}"
        echo -e "${YELLOW}Creating a simple alias for docker compose...${NC}"
        # Create an alias for docker-compose using docker compose
        alias docker-compose="docker compose"
    fi
fi

# Step 1: Set up the sandbox environment
echo -e "${YELLOW}Step 1: Setting up the sandbox environment...${NC}"
./setup_sandbox_local.sh

# Step 2: Run the API tests
echo -e "${YELLOW}Step 2: Running the API tests...${NC}"
./test_sandbox_api.sh

echo -e "${GREEN}Demo completed successfully!${NC}"
echo -e "${GREEN}The Datafold Sandbox API Environment is set up and working correctly.${NC}"
echo ""
echo -e "${YELLOW}To clean up the environment:${NC}"
echo "docker-compose -f docker-compose-local.yml down"
echo "docker network rm datafold_internal_network"
