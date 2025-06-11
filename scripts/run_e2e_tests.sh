#!/bin/bash
#
# End-to-End Integration Test Runner for DataFold Message Signing Authentication
#
# This script orchestrates the complete E2E testing process including:
# - Server startup
# - SDK environment setup
# - Test execution
# - Result reporting
# - Cleanup

set -euo pipefail

# Configuration
E2E_SERVER_URL="${E2E_SERVER_URL:-http://localhost:8080}"
E2E_TIMEOUT_SECS="${E2E_TIMEOUT_SECS:-60}"
E2E_CONCURRENT_CLIENTS="${E2E_CONCURRENT_CLIENTS:-10}"
E2E_ENABLE_ATTACK_SIM="${E2E_ENABLE_ATTACK_SIM:-false}"
E2E_TEST_TYPE="${E2E_TEST_TYPE:-quick}"
E2E_GENERATE_REPORTS="${E2E_GENERATE_REPORTS:-true}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
DataFold E2E Integration Test Runner

Usage: $0 [OPTIONS]

OPTIONS:
    -h, --help                Show this help message
    -t, --type TYPE          Test type: quick, comprehensive, or custom (default: quick)
    -s, --server URL         Server URL (default: http://localhost:8080)
    -T, --timeout SECS       Test timeout in seconds (default: 60)
    -c, --clients NUM        Number of concurrent clients (default: 10)
    -a, --attack-sim         Enable attack simulation tests
    -r, --reports            Generate test reports (default: true)
    --start-server           Start DataFold server before tests
    --stop-server            Stop DataFold server after tests
    --setup-sdks             Setup JavaScript and Python SDK environments
    --cleanup                Clean up test artifacts

EXAMPLES:
    # Run quick E2E tests
    $0 --type quick

    # Run comprehensive tests with attack simulation
    $0 --type comprehensive --attack-sim

    # Run tests with custom configuration
    $0 --type custom --server http://localhost:9000 --clients 20

    # Setup and run full test suite
    $0 --setup-sdks --start-server --type comprehensive --stop-server

ENVIRONMENT VARIABLES:
    E2E_SERVER_URL           DataFold server URL
    E2E_TIMEOUT_SECS         Test timeout in seconds
    E2E_CONCURRENT_CLIENTS   Number of concurrent test clients
    E2E_ENABLE_ATTACK_SIM    Enable attack simulation (true/false)
    E2E_TEST_TYPE            Test type (quick/comprehensive/custom)
    E2E_GENERATE_REPORTS     Generate test reports (true/false)

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -t|--type)
                E2E_TEST_TYPE="$2"
                shift 2
                ;;
            -s|--server)
                E2E_SERVER_URL="$2"
                shift 2
                ;;
            -T|--timeout)
                E2E_TIMEOUT_SECS="$2"
                shift 2
                ;;
            -c|--clients)
                E2E_CONCURRENT_CLIENTS="$2"
                shift 2
                ;;
            -a|--attack-sim)
                E2E_ENABLE_ATTACK_SIM="true"
                shift
                ;;
            -r|--reports)
                E2E_GENERATE_REPORTS="true"
                shift
                ;;
            --start-server)
                START_SERVER="true"
                shift
                ;;
            --stop-server)
                STOP_SERVER="true"
                shift
                ;;
            --setup-sdks)
                SETUP_SDKS="true"
                shift
                ;;
            --cleanup)
                CLEANUP="true"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust and Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi

    # Check Node.js and npm (for JavaScript SDK tests)
    if ! command -v node &> /dev/null; then
        log_warning "Node.js not found. JavaScript SDK tests will be skipped."
    fi

    # Check Python and pip (for Python SDK tests)
    if ! command -v python &> /dev/null && ! command -v python3 &> /dev/null; then
        log_warning "Python not found. Python SDK tests will be skipped."
    fi

    log_success "Prerequisites check completed"
}

# Setup SDK environments
setup_sdks() {
    if [[ "${SETUP_SDKS:-false}" == "true" ]]; then
        log_info "Setting up SDK environments..."

        # Setup JavaScript SDK
        if [[ -d "js-sdk" ]] && command -v npm &> /dev/null; then
            log_info "Setting up JavaScript SDK..."
            cd js-sdk
            npm install
            cd ..
            log_success "JavaScript SDK setup completed"
        fi

        # Setup Python SDK
        if [[ -d "python-sdk" ]] && (command -v python &> /dev/null || command -v python3 &> /dev/null); then
            log_info "Setting up Python SDK..."
            cd python-sdk
            if command -v python3 &> /dev/null; then
                python3 -m pip install -r requirements-dev.txt
            else
                python -m pip install -r requirements-dev.txt
            fi
            cd ..
            log_success "Python SDK setup completed"
        fi
    fi
}

# Start DataFold server
start_server() {
    if [[ "${START_SERVER:-false}" == "true" ]]; then
        log_info "Starting DataFold server..."
        
        # Build the server
        cargo build --bin datafold_http_server
        
        # Start server in background
        cargo run --bin datafold_http_server &
        SERVER_PID=$!
        
        # Wait for server to start
        log_info "Waiting for server to start..."
        for i in {1..30}; do
            if curl -s "${E2E_SERVER_URL}/api/system/status" > /dev/null 2>&1; then
                log_success "DataFold server started successfully"
                return 0
            fi
            sleep 1
        done
        
        log_error "Failed to start DataFold server"
        exit 1
    fi
}

# Stop DataFold server
stop_server() {
    if [[ "${STOP_SERVER:-false}" == "true" ]] && [[ -n "${SERVER_PID:-}" ]]; then
        log_info "Stopping DataFold server..."
        kill "${SERVER_PID}" 2>/dev/null || true
        wait "${SERVER_PID}" 2>/dev/null || true
        log_success "DataFold server stopped"
    fi
}

# Run E2E tests
run_tests() {
    log_info "Running E2E tests..."
    log_info "Configuration:"
    log_info "  Test Type: ${E2E_TEST_TYPE}"
    log_info "  Server URL: ${E2E_SERVER_URL}"
    log_info "  Timeout: ${E2E_TIMEOUT_SECS}s"
    log_info "  Concurrent Clients: ${E2E_CONCURRENT_CLIENTS}"
    log_info "  Attack Simulation: ${E2E_ENABLE_ATTACK_SIM}"

    # Export environment variables for tests
    export E2E_SERVER_URL
    export E2E_TIMEOUT_SECS
    export E2E_CONCURRENT_CLIENTS
    export E2E_ENABLE_ATTACK_SIM

    case "${E2E_TEST_TYPE}" in
        "quick")
            log_info "Running quick E2E tests..."
            cargo test --test e2e_integration_test test_e2e_integration_suite_quick -- --ignored
            ;;
        "comprehensive")
            log_info "Running comprehensive E2E tests..."
            cargo test --test e2e_integration_test test_e2e_integration_suite_comprehensive -- --ignored
            ;;
        "custom")
            log_info "Running custom E2E test configuration..."
            cargo test --test e2e_integration_test -- --ignored
            ;;
        *)
            log_error "Unknown test type: ${E2E_TEST_TYPE}"
            exit 1
            ;;
    esac
}

# Generate test reports
generate_reports() {
    if [[ "${E2E_GENERATE_REPORTS}" == "true" ]]; then
        log_info "Generating test reports..."
        
        # Create reports directory
        mkdir -p test_reports/e2e
        
        # Generate JUnit XML for CI/CD
        if [[ -f "target/debug/deps/test_results.xml" ]]; then
            cp "target/debug/deps/test_results.xml" "test_reports/e2e/junit.xml"
            log_success "JUnit XML report generated: test_reports/e2e/junit.xml"
        fi
        
        # Generate coverage report if available
        if command -v grcov &> /dev/null; then
            log_info "Generating coverage report..."
            grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o test_reports/e2e/coverage/
            log_success "Coverage report generated: test_reports/e2e/coverage/"
        fi
    fi
}

# Cleanup test artifacts
cleanup() {
    if [[ "${CLEANUP:-false}" == "true" ]]; then
        log_info "Cleaning up test artifacts..."
        
        # Remove temporary test data
        rm -rf /tmp/datafold_e2e_test_*
        
        # Clean cargo test artifacts
        cargo clean
        
        log_success "Cleanup completed"
    fi
}

# Main execution
main() {
    log_info "DataFold E2E Integration Test Runner"
    log_info "====================================="
    
    parse_args "$@"
    check_prerequisites
    setup_sdks
    start_server
    
    # Run tests with error handling
    if run_tests; then
        log_success "E2E tests completed successfully"
        TEST_RESULT=0
    else
        log_error "E2E tests failed"
        TEST_RESULT=1
    fi
    
    generate_reports
    stop_server
    cleanup
    
    if [[ $TEST_RESULT -eq 0 ]]; then
        log_success "🎉 All E2E integration tests passed!"
    else
        log_error "❌ E2E integration tests failed"
    fi
    
    exit $TEST_RESULT
}

# Handle script interruption
trap 'log_warning "Script interrupted"; stop_server; exit 130' INT TERM

# Run main function
main "$@"