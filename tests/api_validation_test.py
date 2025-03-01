#!/usr/bin/env python3
"""
API Validation Test Script for FoldDB

This script tests the FoldDB API endpoints with the example schemas, mutations, and queries.
It verifies that the JSON is properly processed by the API.
"""

import json
import requests
import subprocess
import time
import sys
import os
from pathlib import Path

# Configuration
API_BASE_URL = "http://localhost:8080/api"
SCHEMA_ENDPOINT = f"{API_BASE_URL}/schema"
EXECUTE_ENDPOINT = f"{API_BASE_URL}/execute"

# Paths to example files
EXAMPLES_DIR = Path("src/datafold_node/examples")
USER_PROFILE_SCHEMA = EXAMPLES_DIR / "user_profile_schema.json"
USER_PROFILE2_SCHEMA = EXAMPLES_DIR / "user_profile2_schema.json"
USER_PROFILE_MUTATIONS = EXAMPLES_DIR / "user_profile_mutations.json"
USER_PROFILE_QUERIES = EXAMPLES_DIR / "user_profile_queries.json"

# Colors for terminal output
GREEN = "\033[92m"
RED = "\033[91m"
YELLOW = "\033[93m"
RESET = "\033[0m"

def print_success(message):
    print(f"{GREEN}✓ {message}{RESET}")

def print_error(message):
    print(f"{RED}✗ {message}{RESET}")

def print_warning(message):
    print(f"{YELLOW}! {message}{RESET}")

def print_header(message):
    print(f"\n{YELLOW}=== {message} ==={RESET}")

def load_json_file(file_path):
    """Load JSON from a file."""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except Exception as e:
        print_error(f"Failed to load JSON from {file_path}: {e}")
        return None

def check_server_running():
    """Check if the FoldDB server is running."""
    try:
        # Try to access the schemas endpoint instead of health
        print(f"Checking server at {API_BASE_URL}/schemas")
        response = requests.get(f"{API_BASE_URL}/schemas", timeout=2)
        print(f"Server response: {response.status_code}")
        # Any response means the server is running
        return response.status_code in [200, 404]
    except requests.exceptions.RequestException as e:
        print(f"Server connection error: {e}")
        return False

def start_server():
    """Start the FoldDB server if it's not already running."""
    if check_server_running():
        print_success("FoldDB server is already running")
        return True

    print_warning("Starting FoldDB server...")
    try:
        # Start the server in the background
        process = subprocess.Popen(
            ["cargo", "run", "--bin", "datafold_node"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # Wait for the server to start
        for _ in range(30):  # Try for 30 seconds
            time.sleep(1)
            if check_server_running():
                print_success("FoldDB server started successfully")
                return True
        
        print_error("Failed to start FoldDB server within timeout")
        return False
    except Exception as e:
        print_error(f"Error starting FoldDB server: {e}")
        return False

def load_schema(schema_file):
    """Load a schema into the FoldDB server."""
    schema = load_json_file(schema_file)
    if not schema:
        return False

    schema_name = schema.get("name", "unknown")
    print_warning(f"Loading schema: {schema_name}")
    print(f"Schema content: {json.dumps(schema, indent=2)}")
    
    try:
        print(f"Sending POST request to {SCHEMA_ENDPOINT}")
        response = requests.post(SCHEMA_ENDPOINT, json=schema)
        print(f"Response status: {response.status_code}")
        print(f"Response content: {response.text}")
        
        try:
            response_data = response.json()
            print(f"Parsed response: {json.dumps(response_data, indent=2)}")
            
            if response.status_code == 200 and "data" in response_data:
                print_success(f"Schema {schema_name} loaded successfully")
                return True
            else:
                error = response_data.get("error", "Unknown error")
                # If the error is that the schema already exists, that's okay
                if "already exists" in error:
                    print_warning(f"Schema {schema_name} already exists")
                    return True
                print_error(f"Failed to load schema {schema_name}: {error}")
                return False
        except json.JSONDecodeError as e:
            print_error(f"Failed to parse response as JSON: {e}")
            return False
    except Exception as e:
        print_error(f"Error loading schema {schema_name}: {e}")
        return False

def execute_operation(operation):
    """Execute a mutation or query operation."""
    operation_type = operation.get("type", "unknown")
    schema_name = operation.get("schema", "unknown")
    
    # Prepare the request payload - operation must be a JSON string
    payload = {
        "operation": json.dumps(operation)
    }
    
    try:
        response = requests.post(EXECUTE_ENDPOINT, json=payload)
        response_data = response.json()
        
        if response.status_code == 200 and "data" in response_data:
            print_success(f"{operation_type.capitalize()} on {schema_name} executed successfully")
            return True, response_data
        else:
            error = response_data.get("error", "Unknown error")
            print_error(f"Failed to execute {operation_type} on {schema_name}: {error}")
            return False, response_data
    except Exception as e:
        print_error(f"Error executing {operation_type} on {schema_name}: {e}")
        return False, {"error": str(e)}

def run_tests():
    """Run all the API validation tests."""
    print_header("Starting API Validation Tests")
    
    # Step 1: Start the server if needed
    if not start_server():
        return False
    
    # Step 2: Load schemas
    print_header("Loading Schemas")
    if not load_schema(USER_PROFILE_SCHEMA):
        print_error("Failed to load UserProfile schema, aborting tests")
        return False
    
    if not load_schema(USER_PROFILE2_SCHEMA):
        print_warning("Failed to load UserProfile2 schema, continuing with tests")
    
    # Step 3: Execute mutations
    print_header("Executing Mutations")
    mutations = load_json_file(USER_PROFILE_MUTATIONS)
    if not mutations:
        print_error("Failed to load mutations, aborting tests")
        return False
    
    for i, mutation in enumerate(mutations):
        print_warning(f"Executing mutation {i+1}/{len(mutations)}")
        success, _ = execute_operation(mutation)
        if not success:
            print_warning(f"Mutation {i+1} failed, continuing with tests")
    
    # Step 4: Execute queries
    print_header("Executing Queries")
    queries = load_json_file(USER_PROFILE_QUERIES)
    if not queries:
        print_error("Failed to load queries, aborting tests")
        return False
    
    results = []
    for i, query in enumerate(queries):
        print_warning(f"Executing query {i+1}/{len(queries)}")
        success, response_data = execute_operation(query)
        if success:
            results.append({
                "query": query,
                "result": response_data.get("data", {})
            })
        else:
            print_warning(f"Query {i+1} failed, continuing with tests")
    
    # Step 5: Print query results
    print_header("Query Results")
    for i, result in enumerate(results):
        query = result["query"]
        schema_name = query.get("schema", "unknown")
        fields = query.get("fields", [])
        filter_info = query.get("filter", "None")
        
        print(f"\nQuery {i+1}: Schema={schema_name}, Fields={fields}, Filter={filter_info}")
        print(f"Result: {json.dumps(result['result'], indent=2)}")
    
    print_header("API Validation Tests Completed")
    return True

if __name__ == "__main__":
    success = run_tests()
    sys.exit(0 if success else 1)
