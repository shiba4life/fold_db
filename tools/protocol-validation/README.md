# DataFold Protocol Compliance Validation Tools

This directory contains comprehensive validation tools for DataFold's RFC 9421 HTTP Message Signatures implementation. These tools ensure protocol compliance, security, and interoperability across all DataFold implementations.

## Directory Structure

```
tools/protocol-validation/
├── README.md                          # This file
├── Cargo.toml                         # Rust validation tools dependencies
├── package.json                       # Node.js validation tools dependencies
├── requirements.txt                   # Python validation tools dependencies
├── validate.sh                        # Main validation runner script
├── src/                              # Core validation library (Rust)
│   ├── lib.rs                        # Main validation library
│   ├── rfc9421/                      # RFC 9421 compliance validators
│   ├── cross_platform/               # Cross-platform validation
│   ├── security/                     # Security testing tools
│   ├── performance/                  # Performance validation
│   └── test_vectors/                 # Test vector management
├── js/                               # JavaScript validation tools
│   ├── src/                          # TypeScript validation tools
│   └── tests/                        # JavaScript-specific tests
├── python/                           # Python validation tools
│   ├── src/                          # Python validation modules
│   └── tests/                        # Python-specific tests
├── test-vectors/                     # RFC 9421 test vectors
│   ├── rfc9421-compliance/           # Standard compliance vectors
│   ├── edge-cases/                   # Edge case test vectors
│   ├── security-tests/               # Security-focused vectors
│   └── interop-tests/                # Cross-platform test vectors
├── config/                           # Validation configurations
│   ├── compliance.yaml               # RFC 9421 compliance settings
│   ├── security.yaml                 # Security validation settings
│   └── performance.yaml              # Performance test configuration
├── scripts/                          # Validation and testing scripts
│   ├── run-compliance-tests.sh       # Run RFC 9421 compliance tests
│   ├── run-security-tests.sh         # Run security validation
│   ├── run-performance-tests.sh      # Run performance benchmarks
│   ├── generate-test-vectors.sh      # Generate new test vectors
│   └── ci-validation.sh              # CI/CD integration script
└── reports/                          # Validation report templates
    ├── compliance-report.md          # Compliance test report
    ├── security-report.md            # Security validation report
    └── performance-report.md         # Performance benchmark report
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Python 3.9+
- DataFold server running (for integration tests)

### Installation

```bash
# Install all dependencies
./scripts/install-deps.sh

# Or install per platform:
cargo build --release                 # Rust tools
npm install                          # JavaScript tools
pip install -r requirements.txt     # Python tools
```

### Running Validation

```bash
# Run complete validation suite
./validate.sh

# Run specific validation categories
./scripts/run-compliance-tests.sh    # RFC 9421 compliance
./scripts/run-security-tests.sh      # Security validation
./scripts/run-performance-tests.sh   # Performance benchmarks

# Run cross-platform validation
./scripts/run-interop-tests.sh       # All platforms
```

## Validation Categories

### 1. RFC 9421 Compliance Validation

Validates adherence to RFC 9421 HTTP Message Signatures:

- **Header Format Validation**: Signature-Input and Signature header compliance
- **Canonical Message Construction**: Message canonicalization accuracy
- **Signature Component Validation**: Required and optional component handling
- **Test Vector Verification**: Standard and custom test vectors

### 2. Cross-Platform Validation

Ensures consistency across Rust, JavaScript, and Python implementations:

- **Signature Interoperability**: Cross-platform signature verification
- **Message Canonicalization**: Consistent canonical message generation
- **Error Handling**: Uniform error responses and handling
- **Configuration Compatibility**: Consistent configuration behavior

### 3. Security Validation

Comprehensive security testing and attack simulation:

- **Timestamp Validation**: Time window and clock skew testing
- **Nonce Replay Protection**: Replay attack simulation and detection
- **Attack Scenario Testing**: Signature forgery, manipulation attempts
- **Security Parameter Validation**: Cryptographic parameter compliance

### 4. Performance Validation

Performance and scalability testing:

- **Signature Performance**: Generation and verification benchmarks
- **Load Testing**: High-throughput authentication testing
- **Memory Usage**: Resource consumption validation
- **Scalability Testing**: Performance under various loads

## Configuration

### RFC 9421 Compliance Settings

```yaml
# config/compliance.yaml
rfc9421:
  required_components:
    - "@method"
    - "@target-uri"
    - "content-type"
    - "content-digest"
  signature_algorithms:
    - "ed25519"
  header_validation:
    strict_parsing: true
    case_sensitivity: true
```

### Security Validation Settings

```yaml
# config/security.yaml
security:
  timestamp_tests:
    - valid_current_time
    - expired_timestamp
    - future_timestamp
    - clock_skew_tolerance
  nonce_tests:
    - unique_nonce
    - duplicate_nonce
    - invalid_format
  attack_simulations:
    - replay_attack
    - signature_forgery
    - timestamp_manipulation
```

## Test Vectors

### Standard RFC 9421 Test Vectors

Located in `test-vectors/rfc9421-compliance/`:
- Basic signature generation and verification
- Multiple signature components
- Different HTTP methods and content types
- Error cases and edge conditions

### Security Test Vectors

Located in `test-vectors/security-tests/`:
- Replay attack scenarios
- Timestamp manipulation tests
- Signature forgery attempts
- Nonce collision tests

### Cross-Platform Test Vectors

Located in `test-vectors/interop-tests/`:
- Signatures generated by each platform
- Cross-platform verification tests
- Configuration compatibility tests

## CI/CD Integration

### GitHub Actions Integration

```yaml
# Example workflow step
- name: Run Protocol Validation
  run: |
    cd tools/protocol-validation
    ./scripts/ci-validation.sh
```

### Validation Exit Codes

- `0`: All validations passed
- `1`: RFC 9421 compliance failures
- `2`: Security validation failures
- `3`: Performance validation failures
- `4`: Cross-platform compatibility failures

## Reporting

### Compliance Report

Generated as `reports/compliance-report.md` with:
- RFC 9421 compliance status
- Failed test details
- Recommendations for fixes

### Security Report

Generated as `reports/security-report.md` with:
- Security test results
- Vulnerability assessments
- Attack simulation outcomes

### Performance Report

Generated as `reports/performance-report.md` with:
- Benchmark results
- Performance trends
- Scalability analysis

## Contributing

When adding new validation tests:

1. Add test vectors to appropriate `test-vectors/` subdirectory
2. Implement validation logic in `src/` (Rust), `js/src/` (JS), or `python/src/` (Python)
3. Update configuration files if needed
4. Add documentation to this README
5. Test with `./validate.sh` before submitting

## Troubleshooting

### Common Issues

1. **"Test vector not found"**: Ensure test vectors are generated with `./scripts/generate-test-vectors.sh`
2. **"Platform implementation missing"**: Check that all SDKs are properly installed
3. **"Server connection failed"**: Ensure DataFold server is running for integration tests

### Debug Mode

Run with debug output:
```bash
DEBUG=1 ./validate.sh
RUST_LOG=debug ./scripts/run-compliance-tests.sh
```

For more detailed troubleshooting, see individual tool documentation in their respective directories.