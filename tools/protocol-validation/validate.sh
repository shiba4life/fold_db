#!/bin/bash

# DataFold Protocol Compliance Validation Suite
# Main validation runner script

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPORTS_DIR="${SCRIPT_DIR}/reports"
CONFIG_DIR="${SCRIPT_DIR}/config"
DEFAULT_CONFIG="${CONFIG_DIR}/compliance.yaml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
VERBOSE=false
DEBUG=false
FAIL_FAST=false
CATEGORIES=""
OUTPUT_DIR="${REPORTS_DIR}"
CONFIG_FILE="${DEFAULT_CONFIG}"
PARALLEL=1
TIMEOUT=300

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_success() {
    print_status "${GREEN}" "✅ $1"
}

print_warning() {
    print_status "${YELLOW}" "⚠️  $1"
}

print_error() {
    print_status "${RED}" "❌ $1"
}

print_info() {
    print_status "${BLUE}" "ℹ️  $1"
}

# Function to show usage
show_usage() {
    cat << EOF
DataFold Protocol Compliance Validation Suite

USAGE:
    $0 [OPTIONS] [COMMAND]

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -d, --debug             Enable debug logging
    --fail-fast             Stop on first failure
    -c, --config FILE       Configuration file (default: ${DEFAULT_CONFIG})
    -o, --output DIR        Output directory for reports (default: ${OUTPUT_DIR})
    --categories LIST       Comma-separated list of categories to run
    --parallel N            Number of parallel workers (default: ${PARALLEL})
    --timeout N             Timeout for tests in seconds (default: ${TIMEOUT})

COMMANDS:
    all                     Run all validation tests (default)
    rfc9421                 Run only RFC 9421 compliance tests
    security                Run only security validation tests
    cross-platform          Run only cross-platform validation tests
    performance             Run only performance validation tests
    generate-vectors        Generate test vectors
    setup                   Install dependencies and setup environment
    clean                   Clean up reports and temporary files

EXAMPLES:
    $0                      # Run all tests with default configuration
    $0 --verbose rfc9421    # Run RFC 9421 tests with verbose output
    $0 --categories rfc9421,security  # Run specific categories
    $0 --config config/strict.yaml all  # Run with custom configuration
    $0 --parallel 4 performance  # Run performance tests with 4 workers

ENVIRONMENT VARIABLES:
    DATAFOLD_SERVER_URL     DataFold server URL for integration tests
    TEST_ENVIRONMENT        Test environment (development, staging, production)
    RUST_LOG               Rust logging level
    DEBUG                  Enable debug mode (same as --debug)

For more information, see: ${SCRIPT_DIR}/README.md
EOF
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo (Rust)")
    fi
    
    # Check Node.js (optional for cross-platform tests)
    if ! command -v node &> /dev/null; then
        print_warning "Node.js not found - JavaScript tests will be skipped"
    fi
    
    # Check Python (optional for cross-platform tests)
    if ! command -v python3 &> /dev/null; then
        print_warning "Python 3 not found - Python tests will be skipped"
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - ${dep}"
        done
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to setup environment
setup_environment() {
    print_info "Setting up validation environment..."
    
    # Create necessary directories
    mkdir -p "${REPORTS_DIR}"
    mkdir -p "${SCRIPT_DIR}/test-vectors"
    
    # Build the validation tools
    print_info "Building validation tools..."
    cd "${SCRIPT_DIR}"
    if [ "${DEBUG}" = true ]; then
        cargo build --bins
    else
        cargo build --release --bins
    fi
    
    # Install JavaScript dependencies if available
    if [ -d "${PROJECT_ROOT}/js-sdk" ] && command -v npm &> /dev/null; then
        print_info "Installing JavaScript SDK dependencies..."
        cd "${PROJECT_ROOT}/js-sdk"
        npm install --silent
    fi
    
    # Install Python dependencies if available
    if [ -d "${PROJECT_ROOT}/python-sdk" ] && command -v pip3 &> /dev/null; then
        print_info "Installing Python SDK dependencies..."
        cd "${PROJECT_ROOT}/python-sdk"
        pip3 install -r requirements.txt --quiet
    fi
    
    cd "${SCRIPT_DIR}"
    print_success "Environment setup completed"
}

# Function to run validation
run_validation() {
    local command=$1
    local binary_path
    
    if [ "${DEBUG}" = true ]; then
        binary_path="${SCRIPT_DIR}/target/debug/validate-protocol"
    else
        binary_path="${SCRIPT_DIR}/target/release/validate-protocol"
    fi
    
    if [ ! -f "${binary_path}" ]; then
        print_error "Validation binary not found. Run with 'setup' command first."
        exit 1
    fi
    
    # Build command arguments
    local args=()
    args+=("--output" "${OUTPUT_DIR}")
    args+=("--parallel" "${PARALLEL}")
    args+=("--timeout" "${TIMEOUT}")
    
    if [ -f "${CONFIG_FILE}" ]; then
        args+=("--config" "${CONFIG_FILE}")
    fi
    
    if [ "${VERBOSE}" = true ]; then
        args+=("--verbose")
    fi
    
    if [ "${DEBUG}" = true ]; then
        args+=("--debug")
    fi
    
    if [ "${FAIL_FAST}" = true ]; then
        args+=("--fail-fast")
    fi
    
    if [ -n "${CATEGORIES}" ]; then
        IFS=',' read -ra CATEGORY_ARRAY <<< "${CATEGORIES}"
        for category in "${CATEGORY_ARRAY[@]}"; do
            args+=("--categories" "${category}")
        done
    fi
    
    # Add command-specific arguments
    case "${command}" in
        "rfc9421")
            args+=("rfc9421")
            ;;
        "security")
            args+=("security")
            ;;
        "cross-platform")
            args+=("cross-platform")
            ;;
        "performance")
            args+=("performance")
            ;;
        "generate-vectors")
            args+=("generate-vectors")
            ;;
        *)
            args+=("all")
            ;;
    esac
    
    print_info "Running validation: ${command}"
    print_info "Command: ${binary_path} ${args[*]}"
    
    # Set environment variables
    export RUST_LOG="${RUST_LOG:-warn}"
    if [ "${DEBUG}" = true ]; then
        export RUST_LOG="debug"
    fi
    
    # Run the validation
    if "${binary_path}" "${args[@]}"; then
        print_success "Validation completed successfully"
        return 0
    else
        local exit_code=$?
        case ${exit_code} in
            1)
                print_error "Validation failed - see reports for details"
                ;;
            2)
                print_error "Validation encountered errors"
                ;;
            *)
                print_error "Validation failed with exit code ${exit_code}"
                ;;
        esac
        return ${exit_code}
    fi
}

# Function to clean up
cleanup() {
    print_info "Cleaning up..."
    
    # Remove reports
    if [ -d "${REPORTS_DIR}" ]; then
        rm -rf "${REPORTS_DIR}"
        print_info "Removed reports directory"
    fi
    
    # Remove build artifacts
    if [ -d "${SCRIPT_DIR}/target" ]; then
        rm -rf "${SCRIPT_DIR}/target"
        print_info "Removed build artifacts"
    fi
    
    # Remove temporary files
    find "${SCRIPT_DIR}" -name "*.tmp" -delete 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Function to show validation summary
show_summary() {
    if [ -f "${OUTPUT_DIR}/validation-summary.txt" ]; then
        print_info "Validation Summary:"
        cat "${OUTPUT_DIR}/validation-summary.txt"
    fi
    
    if [ -f "${OUTPUT_DIR}/validation-report.json" ]; then
        print_info "JSON report available: ${OUTPUT_DIR}/validation-report.json"
    fi
    
    if [ -f "${OUTPUT_DIR}/validation-report.html" ]; then
        print_info "HTML report available: ${OUTPUT_DIR}/validation-report.html"
        
        # Try to open HTML report in browser
        if command -v open &> /dev/null; then
            print_info "Opening HTML report in browser..."
            open "${OUTPUT_DIR}/validation-report.html"
        elif command -v xdg-open &> /dev/null; then
            print_info "Opening HTML report in browser..."
            xdg-open "${OUTPUT_DIR}/validation-report.html"
        fi
    fi
}

# Parse command line arguments
COMMAND="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--debug)
            DEBUG=true
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        -c|--config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --categories)
            CATEGORIES="$2"
            shift 2
            ;;
        --parallel)
            PARALLEL="$2"
            shift 2
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        all|rfc9421|security|cross-platform|performance|generate-vectors|setup|clean)
            COMMAND="$1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Set debug mode from environment if not already set
if [ "${DEBUG:-}" = "true" ] || [ "${DEBUG:-}" = "1" ]; then
    DEBUG=true
fi

# Main execution
print_info "DataFold Protocol Compliance Validation Suite"
print_info "=============================================="

case "${COMMAND}" in
    "setup")
        check_prerequisites
        setup_environment
        exit 0
        ;;
    "clean")
        cleanup
        exit 0
        ;;
    *)
        check_prerequisites
        
        # Ensure environment is set up
        if [ ! -d "${SCRIPT_DIR}/target" ]; then
            print_info "Setting up environment (first run)..."
            setup_environment
        fi
        
        # Run validation
        if run_validation "${COMMAND}"; then
            show_summary
            exit 0
        else
            exit_code=$?
            show_summary
            exit ${exit_code}
        fi
        ;;
esac