# DataFold Tests

This directory contains top-level tests spanning multiple crates in the DataFold workspace.

## Integration Tests

Location: `integration_tests/`

Files:

- `datafold_node_tests.rs` – tests for DataFold node functionality  
- `schema_field_mapping_tests.rs` – tests for schema field mapping  
- `versioning_tests.rs` – tests for versioning functionality  

## Network Tests

- `network_tests.rs` – tests for network functionality between nodes  

## Test Data

- Directory: `test_data/` – helper functions and test data used by the integration and network tests  

## Running Tests

To run all tests across the workspace:

```bash
cargo test --workspace
```

To run only the integration tests:

```bash
cargo test --test integration_tests
```

To run only the network tests:

```bash
cargo test --test network_tests
