#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting DataFold Node server...${NC}"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Node.js is not installed. Please install Node.js first.${NC}"
    exit 1
fi

# Create data directory if it doesn't exist
mkdir -p data

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
  echo -e "${YELLOW}Installing dependencies...${NC}"
  npm install
fi

# Start the DataFold Node server
echo -e "${GREEN}Starting DataFold Node server on port 8080...${NC}"
node datafold-node-server.js
