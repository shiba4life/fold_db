#!/bin/bash

# Script to run network discovery tests for FoldDB

# Default options
SHOW_OUTPUT=true
RUN_BASIC_ONLY=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --basic)
      RUN_BASIC_ONLY=true
      shift
      ;;
    --no-output)
      SHOW_OUTPUT=false
      shift
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --basic      Run only basic tests that don't require network discovery"
      echo "  --no-output  Don't show test output (removes --nocapture flag)"
      echo "  --help       Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

echo "Running network discovery tests..."

# Set up the output flag
OUTPUT_FLAG=""
if [ "$SHOW_OUTPUT" = true ]; then
  OUTPUT_FLAG="-- --nocapture"
fi

# Run the appropriate tests
if [ "$RUN_BASIC_ONLY" = true ]; then
  echo "Running basic tests only (no network discovery required)"
  cargo test unit_tests::network_discovery_tests::test_discovery_initialization $OUTPUT_FLAG
  cargo test unit_tests::network_discovery_tests::test_node_discovery_disabled $OUTPUT_FLAG
  cargo test unit_tests::network_discovery_tests::test_manual_node_connection $OUTPUT_FLAG
else
  echo "Running all network discovery tests"
  cargo test unit_tests::network_discovery_tests $OUTPUT_FLAG
fi

echo "Tests completed."
