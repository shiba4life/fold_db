# Fold Node Tests

This directory contains unit tests for the `fold_node` crate.

## Test Organization

- **atom_tests.rs**: Tests for the Atom and AtomRef functionality
- **schema_tests.rs**: Tests for the Schema system
- **schema_interpreter_tests.rs**: Tests for the Schema interpreter
- **folddb_tests.rs**: Tests for the FoldDB core functionality
- **node_mutation_tests.rs**: Tests for node mutation operations
- **permission_tests.rs**: Tests for permission-based access
- **permissions_tests.rs**: Tests for the permission wrapper
- **network_tests.rs**: Tests for network initialization and operations
- **network_discovery_tests.rs**: Tests for network discovery
- **request_forwarding_tests.rs**: Tests for request forwarding between nodes
- **app_server_tests.rs**: Tests for the application server
- **test_data/**: Helper functions and test data used by multiple tests

## Running Tests

To run all tests for the fold_node crate:

```bash
cd fold_node
cargo test
```

To run a specific test:

```bash
cargo test --package fold_node test_schema_creation
```

## Test Data

The `test_data` directory contains helper functions for creating test schemas and other test data. These are used by multiple test files to ensure consistent test data.

### Test Helpers

The `test_data/test_helpers` directory contains utility functions for:

- **operation_builder.rs**: Creating test queries and mutations
- **schema_builder.rs**: Creating test schemas with specific permissions
