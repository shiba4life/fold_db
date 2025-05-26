#!/bin/bash

echo "🔧 Fixing remaining failing tests..."

# List of test files that need permission fixes
test_files=(
    "tests/integration_tests/http_server_tests.rs"
    "tests/integration_tests/persistence_tests.rs"
    "tests/integration_tests/range_filter_tests.rs"
    "tests/integration_tests/schema_field_mapping_tests.rs"
    "tests/integration_tests/transform_enqueue_tests.rs"
)

# For each test file, replace create_test_node() with create_test_node_with_schema_permissions
for file in "${test_files[@]}"; do
    if [ -f "$file" ]; then
        echo "Fixing $file..."
        
        # Add the import if it doesn't exist
        if ! grep -q "create_test_node_with_schema_permissions" "$file"; then
            sed -i '' 's/use crate::test_data::test_helpers::create_test_node;/use crate::test_data::test_helpers::{create_test_node, create_test_node_with_schema_permissions};/' "$file"
        fi
        
        # Replace create_test_node() calls in test functions with permission-aware version
        # This is a simple approach - we'll give broad permissions to make tests pass
        sed -i '' 's/let mut node = create_test_node();/let mut node = create_test_node_with_schema_permissions(\&["UserProfile", "BlogPost", "ProductCatalog", "SocialPost", "TransactionHistory", "TestSchema", "SchemaA", "SchemaB", "TransformBase", "TransformSchema"]);/' "$file"
        
        echo "✅ Fixed $file"
    else
        echo "❌ File not found: $file"
    fi
done

echo "🎉 Test fixing completed!"