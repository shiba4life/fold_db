#!/bin/bash

# Set colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting DataFold Node Persistence Test${NC}"
echo -e "${YELLOW}This test will:${NC}"
echo -e "1. Start the DataFold Node if it's not running"
echo -e "2. Create a Post schema if it doesn't exist"
echo -e "3. Create test posts in the database"
echo -e "4. Verify the posts were created in memory"
echo -e "5. Verify the posts were saved to disk"
echo -e "6. Clean up after the test"
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Node.js is not installed. Please install Node.js to run this test.${NC}"
    exit 1
fi

# Check if the test file exists
if [ ! -f "test/test-datafold-node-persistence.js" ]; then
    echo -e "${RED}Test file not found: test/test-datafold-node-persistence.js${NC}"
    exit 1
fi

# Run the test
echo -e "${GREEN}Running test...${NC}"
node test/test-datafold-node-persistence.js

# Check if the test was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}Test completed successfully!${NC}"
else
    echo -e "${RED}Test failed!${NC}"
    exit 1
fi

exit 0
