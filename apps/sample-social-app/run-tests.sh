#!/bin/bash

# Social App Test Runner Script
# This script provides a convenient way to run the social app tests

# Make script executable
chmod +x test-runner.js

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print header
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}       Social App Test Runner           ${NC}"
echo -e "${BLUE}=========================================${NC}"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed${NC}"
    echo "Please install Node.js to run the tests"
    exit 1
fi

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    npm install
    echo ""
fi

# Parse arguments
HEADLESS=false
SUITE="all"

for arg in "$@"; do
    case $arg in
        --headless)
            HEADLESS=true
            ;;
        --suite=*)
            SUITE="${arg#*=}"
            ;;
        --help)
            echo -e "${GREEN}Usage:${NC}"
            echo "  ./run-tests.sh [options]"
            echo ""
            echo -e "${GREEN}Options:${NC}"
            echo "  --headless         Run tests in headless mode"
            echo "  --suite=SUITE      Run specific test suite (navigation, post, profile, friend, all)"
            echo "  --help             Show this help message"
            exit 0
            ;;
    esac
done

# Print test configuration
echo -e "${GREEN}Test Configuration:${NC}"
echo "  Suite: $SUITE"
echo "  Headless: $HEADLESS"
echo ""

# Build command
CMD="node test-runner.js"

if [ "$HEADLESS" = true ]; then
    CMD="$CMD --headless"
fi

if [ "$SUITE" != "all" ]; then
    CMD="$CMD --suite=$SUITE"
fi

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
echo "Command: $CMD"
echo -e "${BLUE}=========================================${NC}"
echo ""

# Execute the command
eval $CMD

# Check exit code
EXIT_CODE=$?
echo ""
echo -e "${BLUE}=========================================${NC}"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}Tests completed successfully!${NC}"
else
    echo -e "${RED}Tests failed with exit code $EXIT_CODE${NC}"
fi

exit $EXIT_CODE
