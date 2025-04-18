# DataFold Integration Tests

This directory contains integration tests that span multiple crates in the DataFold project.

## Test Organization

- **integration_tests/**: Tests that verify the integration between different components
  - **datafold_node_tests.rs**: Tests for the DataFold node functionality
  - **schema_field_mapping_tests.rs**: Tests for schema field mapping
  - **schema_mapping_tests.rs**: Tests for schema mapping
  - **user_profile_api_tests.rs**: Tests for the user profile API
  - **versioning_tests.rs**: Tests for versioning functionality
  - **web_api_tests.rs**: Tests for the web API

- **test_data/**: Helper functions and test data used by multiple tests

- **network_tests.rs**: Tests for the network functionality between nodes
- **test_query_fix.rs**: Tests for query functionality

## Test Reorganization

The tests have been reorganized to place them in their appropriate crates:

### Crate-Specific Tests

Each crate now has its own tests directory:

- **fold_node/tests/**: Tests specific to the node functionality
  - **atom_tests.rs**: Tests for Atom and AtomRef
  - **schema_tests.rs**: Tests for Schema system
  - **schema_interpreter_tests.rs**: Tests for Schema interpreter
  - **folddb_tests.rs**: Tests for FoldDB core
  - **node_mutation_tests.rs**: Tests for node mutations
  - **permission_tests.rs**: Tests for permission-based access
  - **permissions_tests.rs**: Tests for permission wrapper
  - **network_tests.rs**: Tests for network initialization
  - **network_discovery_tests.rs**: Tests for network discovery
  - **request_forwarding_tests.rs**: Tests for request forwarding
  - **app_server_tests.rs**: Tests for application server

### Top-Level Integration Tests

The top-level tests directory contains only integration tests that span multiple crates:

- **integration_tests/**: Tests that verify integration between components
- **network_tests.rs**: Tests for network functionality between nodes
- **test_query_fix.rs**: Tests for query functionality

## Running Tests

To run all tests across the workspace:

```bash
cargo test --workspace
```

To run only the integration tests:

```bash
cargo test --test integration_tests
```

To run a specific integration test:

```bash
cargo test --test integration_tests::datafold_node_tests
```

To run tests for a specific crate:

```bash
cargo test --package fold_node
```
