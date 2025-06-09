# DataFold Python SDK

A client-side Python SDK for Ed25519 key generation, management, and server integration.

## Features

- **Ed25519 Key Generation**: Generate cryptographically secure Ed25519 key pairs
- **Cross-Platform Compatibility**: Works on macOS, Windows, and Linux
- **Multiple Key Formats**: Support for hex, base64, bytes, and PEM formats
- **Secure Key Storage**: OS keychain integration with encrypted file fallback
- **Client-Side Security**: Private keys never leave the client environment
- **Comprehensive Validation**: Built-in validation for all cryptographic operations
- **Memory Security**: Best-effort secure memory clearing
- **Professional Package Structure**: Installable via pip with proper dependencies
- **Type Safety**: Full type hints and mypy compatibility
- **Comprehensive Testing**: 90%+ test coverage with pytest

## Installation

```bash
pip install datafold-python-sdk
```

### Development Installation

```bash
git clone https://github.com/datafold/datafold.git
cd datafold/python-sdk
pip install -e ".[dev]"
```

## Quick Start

```python
import datafold_sdk

# Initialize the SDK and check compatibility
result = datafold_sdk.initialize_sdk()
if not result['compatible']:
    print(f"Platform not compatible: {result['warnings']}")
    exit(1)

# Generate an Ed25519 key pair
key_pair = datafold_sdk.generate_key_pair()
print(f"Generated key pair with {len(key_pair.private_key)} byte private key")

# Format keys in different formats
private_hex = datafold_sdk.format_key(key_pair.private_key, 'hex')
public_base64 = datafold_sdk.format_key(key_pair.public_key, 'base64')
private_pem = datafold_sdk.format_key(key_pair.private_key, 'pem')

print(f"Private key (hex): {private_hex}")
print(f"Public key (base64): {public_base64}")
print(f"Private key (PEM):\n{private_pem}")

# Parse keys back from formats
parsed_private = datafold_sdk.parse_key(private_hex, 'hex')
parsed_public = datafold_sdk.parse_key(public_base64, 'base64')

assert parsed_private == key_pair.private_key
assert parsed_public == key_pair.public_key

# Generate multiple key pairs
key_pairs = datafold_sdk.generate_multiple_key_pairs(5)
print(f"Generated {len(key_pairs)} key pairs")

# Clear sensitive material when done
for kp in key_pairs:
    datafold_sdk.clear_key_material(kp)
```

## API Reference

### Key Generation

#### `generate_key_pair(*, validate=True, entropy=None) -> Ed25519KeyPair`

Generates a new Ed25519 key pair using the cryptography package.

**Parameters:**
- `validate` (bool, optional): Whether to validate the generated keys (default: True)
- `entropy` (bytes, optional): Custom entropy source for testing only (32 bytes)

**Returns:** `Ed25519KeyPair` object containing private and public keys

**Raises:**
- `UnsupportedPlatformError`: If cryptography package is not available
- `Ed25519KeyError`: If key generation fails
- `ValidationError`: If validation fails

**Example:**
```python
# Generate with default options
key_pair = datafold_sdk.generate_key_pair()

# Generate without validation (faster)
key_pair = datafold_sdk.generate_key_pair(validate=False)

# Generate with custom entropy (testing only)
import secrets
entropy = secrets.token_bytes(32)
key_pair = datafold_sdk.generate_key_pair(entropy=entropy)
```

#### `generate_multiple_key_pairs(count, *, validate=True) -> List[Ed25519KeyPair]`

Generates multiple Ed25519 key pairs efficiently.

**Parameters:**
- `count` (int): Number of key pairs to generate (1-100)
- `validate` (bool, optional): Whether to validate the generated keys

**Returns:** List of `Ed25519KeyPair` objects

**Example:**
```python
# Generate 10 key pairs
key_pairs = datafold_sdk.generate_multiple_key_pairs(10)

# Generate without validation for speed
key_pairs = datafold_sdk.generate_multiple_key_pairs(5, validate=False)
```

### Key Formatting

#### `format_key(key, format_type) -> Union[str, bytes]`

Converts a key to the specified format.

**Parameters:**
- `key` (bytes): The key to format
- `format_type` (str): Output format ('hex', 'base64', 'bytes', 'pem')

**Returns:** Formatted key (str for hex/base64/pem, bytes for bytes)

**Example:**
```python
key_pair = datafold_sdk.generate_key_pair()

# Format as hex string
hex_key = datafold_sdk.format_key(key_pair.private_key, 'hex')

# Format as base64 string
b64_key = datafold_sdk.format_key(key_pair.public_key, 'base64')

# Format as PEM string
pem_key = datafold_sdk.format_key(key_pair.private_key, 'pem')

# Get bytes copy
key_copy = datafold_sdk.format_key(key_pair.private_key, 'bytes')
```

#### `parse_key(key_data, format_type) -> bytes`

Parses a key from the specified format.

**Parameters:**
- `key_data` (Union[str, bytes]): The key data to parse
- `format_type` (str): Input format ('hex', 'base64', 'bytes', 'pem')

**Returns:** Parsed key as bytes

### Utility Functions

#### `check_platform_compatibility() -> Dict[str, Any]`

Checks platform compatibility for Ed25519 operations.

**Returns:** Dictionary with compatibility information

#### `initialize_sdk() -> Dict[str, Any]`

Initializes the SDK and performs compatibility checks.

**Returns:** Dictionary with 'compatible' (bool) and 'warnings' (list)

#### `is_compatible() -> bool`

Quick synchronous compatibility check.

#### `clear_key_material(key_pair) -> None`

Clears sensitive key material from memory (best effort).

## Data Classes

### `Ed25519KeyPair`

Represents an Ed25519 key pair.

**Attributes:**
- `private_key` (bytes): The private key (32 bytes)
- `public_key` (bytes): The public key (32 bytes)

**Example:**
```python
key_pair = datafold_sdk.generate_key_pair()
print(f"Private key length: {len(key_pair.private_key)}")
print(f"Public key length: {len(key_pair.public_key)}")
```

## Security Features

### Client-Side Key Generation

All key generation happens entirely on the client using the Python `secrets` module and `cryptography` package's secure random number generation. Private keys never leave the client environment.

### Secure Random Generation

The SDK uses Python's `secrets.token_bytes()` for cryptographically secure random number generation, ensuring high-quality entropy for key generation.

### Memory Security

The SDK provides utilities to clear sensitive key material from memory. While Python's garbage collection limits guaranteed memory clearing, the SDK makes a best-effort attempt.

### Platform Validation

The SDK validates that the cryptography package is available and that Ed25519 operations are supported on the current platform.

## Error Handling

All SDK functions raise specific exception types for programmatic handling:

```python
import datafold_sdk
from datafold_sdk import Ed25519KeyError, ValidationError, UnsupportedPlatformError

try:
    key_pair = datafold_sdk.generate_key_pair()
except UnsupportedPlatformError as e:
    print(f"Platform not supported: {e}")
except Ed25519KeyError as e:
    print(f"Key generation failed: {e}")
except ValidationError as e:
    print(f"Validation failed: {e}")
```

## Platform Compatibility

- **Python**: 3.8+ (supports 3.8, 3.9, 3.10, 3.11, 3.12)
- **Operating Systems**: macOS, Windows, Linux
- **Dependencies**: cryptography>=41.0.0

**Requirements:**
- Python 3.8 or higher
- cryptography package (automatically installed)
- Optional: keyring package for secure storage features

## Examples

### Basic Key Generation and Validation

```python
import datafold_sdk

# Check platform compatibility first
if not datafold_sdk.is_compatible():
    print("Platform not compatible with DataFold SDK")
    exit(1)

# Generate and validate a key pair
key_pair = datafold_sdk.generate_key_pair()

# The key pair is automatically validated during generation
print("✓ Generated valid Ed25519 key pair")

# Check key lengths
assert len(key_pair.private_key) == 32
assert len(key_pair.public_key) == 32
print("✓ Key lengths are correct")

# Ensure keys are different
assert key_pair.private_key != key_pair.public_key
print("✓ Private and public keys are different")
```

### Key Format Conversion

```python
import datafold_sdk

key_pair = datafold_sdk.generate_key_pair()

# Convert to different formats
formats = ['hex', 'base64', 'pem']
converted_keys = {}

for fmt in formats:
    private_formatted = datafold_sdk.format_key(key_pair.private_key, fmt)
    public_formatted = datafold_sdk.format_key(key_pair.public_key, fmt)
    
    converted_keys[fmt] = {
        'private': private_formatted,
        'public': public_formatted
    }
    
    print(f"{fmt.upper()} format:")
    print(f"  Private: {private_formatted[:50]}...")
    print(f"  Public:  {public_formatted[:50]}...")

# Convert back and verify
for fmt in formats:
    private_parsed = datafold_sdk.parse_key(converted_keys[fmt]['private'], fmt)
    public_parsed = datafold_sdk.parse_key(converted_keys[fmt]['public'], fmt)
    
    assert private_parsed == key_pair.private_key
    assert public_parsed == key_pair.public_key
    print(f"✓ {fmt.upper()} roundtrip successful")
```

### Batch Key Generation

```python
import datafold_sdk
import time

# Generate multiple keys and measure performance
start_time = time.time()
key_pairs = datafold_sdk.generate_multiple_key_pairs(100)
end_time = time.time()

print(f"Generated {len(key_pairs)} key pairs in {end_time - start_time:.3f} seconds")
print(f"Average: {(end_time - start_time) / len(key_pairs) * 1000:.2f} ms per key pair")

# Verify all keys are unique
private_keys = [kp.private_key for kp in key_pairs]
assert len(set(private_keys)) == len(key_pairs)
print("✓ All generated keys are unique")

# Clean up sensitive material
for key_pair in key_pairs:
    datafold_sdk.clear_key_material(key_pair)
print("✓ Cleared all key material")
```

### Error Handling and Validation

```python
import datafold_sdk
from datafold_sdk import Ed25519KeyError, ValidationError

# Test error handling
try:
    # This should fail - invalid entropy length
    invalid_entropy = b"too_short"
    key_pair = datafold_sdk.generate_key_pair(entropy=invalid_entropy)
except Ed25519KeyError as e:
    print(f"✓ Caught expected error: {e}")

try:
    # This should fail - invalid format
    key_pair = datafold_sdk.generate_key_pair()
    datafold_sdk.format_key(key_pair.private_key, 'invalid_format')
except Ed25519KeyError as e:
    print(f"✓ Caught expected error: {e}")

try:
    # This should fail - invalid key data
    datafold_sdk.parse_key("invalid_hex", 'hex')
except Ed25519KeyError as e:
    print(f"✓ Caught expected error: {e}")
```

### Performance Testing

```python
import datafold_sdk
import time
import statistics

def benchmark_key_generation(count=100):
    """Benchmark key generation performance"""
    times = []
    
    for _ in range(count):
        start = time.time()
        key_pair = datafold_sdk.generate_key_pair()
        end = time.time()
        times.append(end - start)
        
        # Clear to prevent memory buildup
        datafold_sdk.clear_key_material(key_pair)
    
    return {
        'mean': statistics.mean(times) * 1000,  # ms
        'median': statistics.median(times) * 1000,  # ms
        'min': min(times) * 1000,  # ms
        'max': max(times) * 1000,  # ms
        'count': count
    }

# Run benchmark
results = benchmark_key_generation(100)
print(f"Key Generation Benchmark ({results['count']} iterations):")
print(f"  Mean:   {results['mean']:.2f} ms")
print(f"  Median: {results['median']:.2f} ms")
print(f"  Min:    {results['min']:.2f} ms")
print(f"  Max:    {results['max']:.2f} ms")
```

## Testing

The SDK includes comprehensive unit tests with high coverage:

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=src/datafold_sdk --cov-report=html

# Run specific test categories
pytest -m unit
pytest -m integration
```

## Development

### Setting up Development Environment

```bash
# Clone the repository
git clone https://github.com/datafold/datafold.git
cd datafold/python-sdk

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install development dependencies
pip install -e ".[dev]"

# Install pre-commit hooks
pre-commit install

# Run tests
pytest

# Run type checking
mypy src/

# Run code formatting
black src/ tests/
isort src/ tests/

# Run linting
flake8 src/ tests/
```

### Building and Distribution

```bash
# Build the package
python -m build

# Upload to PyPI (maintainers only)
twine upload dist/*
```

## Contributing

See the main DataFold repository for contribution guidelines.

## License

MIT License - see LICENSE file for details.

## Security

This SDK follows security best practices for client-side cryptography:

- All operations are performed client-side
- Private keys never leave the local environment
- Secure random number generation using Python's `secrets` module
- Input validation and comprehensive error handling
- Memory clearing utilities for sensitive data
- Type safety with comprehensive type hints

For security issues, please follow responsible disclosure practices.

## Changelog

### Version 0.1.0 (Initial Release)

- ✅ Ed25519 key generation using cryptography package
- ✅ Multiple key format support (hex, base64, bytes, PEM)
- ✅ Comprehensive validation and error handling
- ✅ Cross-platform compatibility (macOS, Windows, Linux)
- ✅ Memory security utilities
- ✅ Professional packaging with proper dependencies
- ✅ 90%+ test coverage
- ✅ Full type hints and mypy compatibility
- ✅ Security-focused implementation following research guidelines