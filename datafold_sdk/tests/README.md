# DataFold SDK Tests

This directory contains tests for the DataFold SDK.

## Test Structure

The tests are organized into the following files:

- `sdk_tests.rs`: Unit tests for the SDK components
- `real_integration_tests.rs`: Integration tests with a real node

## Running the Tests

You can run the tests using the provided test script:

```bash
cd datafold_sdk
./test_sdk.sh
```

This will:
1. Build the SDK
2. Run all unit tests
3. Run all integration tests
4. Run all SDK tests
5. Build the examples

To also run the examples, use:

```bash
./test_sdk.sh --run-examples
```

## Running Individual Tests

You can also run specific tests using Cargo:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only real integration tests
cargo test --test real_integration_tests

# Run only SDK tests
cargo test --test sdk_tests

# Run a specific test
cargo test test_client_creation

# Run tests with output
cargo test -- --nocapture
```

## Test Coverage

The tests cover the following functionality:

1. **Client Creation and Management**
   - Creating clients with default and custom connections
   - Client properties and methods

2. **Schema Discovery**
   - Discovering local schemas
   - Discovering remote schemas
   - Getting schema details

3. **Query Operations**
   - Basic queries
   - Queries with filters
   - Queries with field selection

4. **Mutation Operations**
   - Create mutations
   - Update mutations
   - Delete mutations

5. **Network Operations**
   - Discovering nodes
   - Checking node availability
   - Getting node information

6. **Container Management**
   - Creating containers
   - Container configuration
   - Container lifecycle

7. **Error Handling**
   - Invalid queries
   - Invalid mutations
   - Non-existent schemas

8. **End-to-End Workflows**
   - Complete CRUD operations
   - Schema discovery and usage
   - Network operations

## Test Environments

### Mock Server (sdk_tests.rs)

The SDK tests use a mock server to simulate the behavior of a real DataFold node. The mock server provides:

- Schema information
- Data storage and retrieval
- Node discovery
- Error handling

This allows testing the SDK without requiring a real DataFold node to be running.

### Real Node Integration (real_integration_tests.rs)

The real integration tests connect to an actual DataFold node instance that is launched during the test. These tests:

- Start a DataFold node process
- Connect to it using the SDK
- Perform real operations against the node
- Verify the results
- Clean up the node process

These tests provide end-to-end validation of the SDK's functionality with a real node.
