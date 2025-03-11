#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting FoldSocial with DataFold node...${NC}"

# Check if DataFold node is running
echo -e "${YELLOW}Checking if DataFold node is running...${NC}"
if curl -s http://localhost:8080/api/schemas > /dev/null; then
  echo -e "${GREEN}DataFold node is already running.${NC}"
else
  echo -e "${YELLOW}DataFold node is not running. Starting it now...${NC}"
  
  # Start the DataFold Node server in the background
  echo -e "${YELLOW}Starting DataFold Node server...${NC}"
  node datafold-node-server.js > datafold-node.log 2>&1 &
  
  # Save the process ID
  NODE_PID=$!
  echo -e "${YELLOW}DataFold Node server started with PID: ${NODE_PID}${NC}"
  
  # Wait for the node to be ready
  echo -e "${YELLOW}Waiting for DataFold node to be ready...${NC}"
  MAX_RETRIES=30
  RETRY_COUNT=0
  NODE_READY=false
  
  while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:8080/api/schemas > /dev/null; then
      NODE_READY=true
      break
    fi
    echo -e "${YELLOW}Waiting for node to start (${RETRY_COUNT}/${MAX_RETRIES})...${NC}"
    RETRY_COUNT=$((RETRY_COUNT+1))
    sleep 2
  done
  
  if [ "$NODE_READY" = true ]; then
    echo -e "${GREEN}DataFold node is ready!${NC}"
  else
    echo -e "${RED}DataFold node did not start properly. Please check datafold-node.log for errors.${NC}"
    exit 1
  fi
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
  echo -e "${YELLOW}Installing dependencies...${NC}"
  npm install
fi

# Run the test script to populate the database with sample posts
echo -e "${YELLOW}Running test script to populate the database with sample posts...${NC}"
node test/test-posts.js

# Start the application
echo -e "${GREEN}Starting FoldSocial application...${NC}"
npm start

# Note: This will keep the DataFold Node server running in the background
# To stop it, you'll need to find and kill the process:
# ps aux | grep datafold-node-server.js
# kill <PID>
