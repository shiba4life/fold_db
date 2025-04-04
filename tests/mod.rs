// Top-level integration tests between different crates
mod test_data;
mod integration_tests;

// These tests are kept at the top level because they test integration between different crates
mod network_tests;
mod test_query_fix;

// Note: Unit tests for specific crates have been moved to their respective crate test directories:
// - fold_node/tests/ for fold_node crate tests (atom_tests, schema_tests, etc.)
// - datafold_sdk/tests/ for datafold_sdk crate tests (sdk_tests, real_integration_tests)
// - fold_client/tests/ for fold_client crate tests (auth_tests, client_tests, etc.)
