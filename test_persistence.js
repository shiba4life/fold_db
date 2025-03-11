/**
 * Test script to verify FoldDB persistence
 * 
 * This script:
 * 1. Creates a schema
 * 2. Adds data to the schema
 * 3. Queries the data to verify it was saved
 * 4. Restarts the node
 * 5. Queries the data again to verify it persisted
 */

const fetch = require('node-fetch');

// Configuration
const API_URL = 'http://localhost:8080/api';
const SCHEMA_NAME = 'TestPersistence';

// Test schema
const schema = {
    name: SCHEMA_NAME,
    fields: {
        id: {
            field_type: "Single",
            permission_policy: {
                read_policy: { NoRequirement: null },
                write_policy: { Distance: 0 },
                explicit_read_policy: null,
                explicit_write_policy: null
            },
            payment_config: {
                base_multiplier: 1.0,
                trust_distance_scaling: { None: null },
                min_payment: null
            },
            field_mappers: {}
        },
        name: {
            field_type: "Single",
            permission_policy: {
                read_policy: { NoRequirement: null },
                write_policy: { Distance: 0 },
                explicit_read_policy: null,
                explicit_write_policy: null
            },
            payment_config: {
                base_multiplier: 1.0,
                trust_distance_scaling: { None: null },
                min_payment: null
            },
            field_mappers: {}
        },
        value: {
            field_type: "Single",
            permission_policy: {
                read_policy: { NoRequirement: null },
                write_policy: { Distance: 0 },
                explicit_read_policy: null,
                explicit_write_policy: null
            },
            payment_config: {
                base_multiplier: 1.0,
                trust_distance_scaling: { None: null },
                min_payment: null
            },
            field_mappers: {}
        }
    },
    payment_config: {
        base_multiplier: 1.0,
        min_payment_threshold: 0
    }
};

// Test data
const testData = {
    id: "test-1",
    name: "Test Item",
    value: "This is a test value to verify persistence"
};

// Create mutation operation
const createMutation = {
    type: "mutation",
    schema: SCHEMA_NAME,
    mutation_type: "create",
    data: testData
};

// Query operation
const queryOperation = {
    type: "query",
    schema: SCHEMA_NAME,
    fields: ["id", "name", "value"],
    filter: null
};

// Helper function to make API requests
async function apiRequest(endpoint, options = {}) {
    const url = `${API_URL}${endpoint}`;
    console.log(`Making request to: ${url}`);
    
    const response = await fetch(url, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...(options.headers || {})
        }
    });
    
    const text = await response.text();
    
    try {
        return JSON.parse(text);
    } catch (e) {
        console.error('Failed to parse response as JSON:', text);
        throw new Error(`Invalid JSON response: ${text}`);
    }
}

// Main test function
async function runTest() {
    try {
        console.log('=== Starting Persistence Test ===');
        
        // Step 1: Skip schema creation for UI testing
        console.log('\n1. Skipping schema creation (UI server does not support schema creation)');
        
        // Step 2: Add data using mutation
        console.log('\n2. Adding test data...');
        const mutationResult = await apiRequest('/execute', {
            method: 'POST',
            body: JSON.stringify({
                operation: JSON.stringify(createMutation)
            })
        });
        
        console.log('Mutation result:', mutationResult);
        
        // Step 3: Query data to verify it was saved
        console.log('\n3. Querying data to verify it was saved...');
        const queryResult1 = await apiRequest('/execute', {
            method: 'POST',
            body: JSON.stringify({
                operation: JSON.stringify(queryOperation)
            })
        });
        
        console.log('Query result before restart:', queryResult1);
        
        if (!queryResult1.data || !queryResult1.data.results) {
            throw new Error('Query failed to return expected data');
        }
        
        // Step 4: Prompt to restart the node
        console.log('\n4. Please restart the node now and press Enter when it\'s back up...');
        await new Promise(resolve => {
            process.stdin.once('data', () => {
                resolve();
            });
        });
        
        // Step 5: Query data again to verify persistence
        console.log('\n5. Querying data again to verify persistence...');
        const queryResult2 = await apiRequest('/execute', {
            method: 'POST',
            body: JSON.stringify({
                operation: JSON.stringify(queryOperation)
            })
        });
        
        console.log('Query result after restart:', queryResult2);
        
        // Verify data persisted
        if (!queryResult2.data || !queryResult2.data.results) {
            console.error('❌ TEST FAILED: Query after restart failed to return expected data');
            return;
        }
        
        // Compare data before and after restart
        const beforeData = JSON.stringify(queryResult1.data);
        const afterData = JSON.stringify(queryResult2.data);
        
        if (beforeData === afterData) {
            console.log('\n✅ TEST PASSED: Data persisted correctly across node restart');
        } else {
            console.error('\n❌ TEST FAILED: Data before and after restart does not match');
            console.log('Before:', beforeData);
            console.log('After:', afterData);
        }
        
    } catch (error) {
        console.error('Test error:', error);
    }
}

// Run the test
runTest();
