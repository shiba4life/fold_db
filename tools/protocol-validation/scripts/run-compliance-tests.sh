#!/bin/bash

# DataFold Protocol RFC 9421 Compliance Test Runner
# This script runs comprehensive RFC 9421 HTTP Message Signatures compliance tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATION_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🔍 Running RFC 9421 Compliance Tests${NC}"
echo "=============================================="

# Ensure validation tools are built
if [ ! -f "${VALIDATION_DIR}/target/release/validate-protocol" ]; then
    echo "Building validation tools..."
    cd "${VALIDATION_DIR}"
    cargo build --release --bin validate-protocol
fi

# Run RFC 9421 compliance tests
echo -e "${YELLOW}📋 Testing header format compliance...${NC}"
echo -e "${YELLOW}📋 Testing canonical message construction...${NC}"
echo -e "${YELLOW}📋 Testing signature component validation...${NC}"
echo -e "${YELLOW}📋 Testing test vector compliance...${NC}"

# Execute the validation
cd "${VALIDATION_DIR}"
./target/release/validate-protocol \
    --verbose \
    --output "${VALIDATION_DIR}/reports" \
    --config "${VALIDATION_DIR}/config/compliance.yaml" \
    rfc9421 \
    --strict-headers \
    --test-vectors "${VALIDATION_DIR}/test-vectors/rfc9421-compliance"

echo -e "${GREEN}✅ RFC 9421 compliance tests completed!${NC}"

# Show report location
if [ -f "${VALIDATION_DIR}/reports/validation-report.html" ]; then
    echo -e "${GREEN}📊 HTML report available: ${VALIDATION_DIR}/reports/validation-report.html${NC}"
fi

if [ -f "${VALIDATION_DIR}/reports/validation-report.json" ]; then
    echo -e "${GREEN}📊 JSON report available: ${VALIDATION_DIR}/reports/validation-report.json${NC}"
fi