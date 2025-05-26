#!/bin/bash

echo "🚀 Testing Schema State Management API Endpoints"
echo "================================================"

BASE_URL="http://localhost:8080/api"

# Function to test an endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    local data=$4
    
    echo ""
    echo "🔍 Testing: $description"
    echo "   $method $endpoint"
    
    if [ -n "$data" ]; then
        response=$(curl -s -X $method "$BASE_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data" \
            -w "\nHTTP_CODE:%{http_code}")
    else
        response=$(curl -s -X $method "$BASE_URL$endpoint" \
            -w "\nHTTP_CODE:%{http_code}")
    fi
    
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        echo "   ✅ Success ($http_code)"
        echo "   📄 Response: $body"
    else
        echo "   ❌ Failed ($http_code)"
        echo "   📄 Response: $body"
    fi
}

# Wait for server to be ready
echo "⏳ Waiting for server to start..."
for i in {1..10}; do
    if curl -s http://localhost:8080/api/schemas > /dev/null 2>&1; then
        echo "✅ Server is ready!"
        break
    fi
    echo "   Attempt $i/10..."
    sleep 2
done

# Test existing endpoints first
test_endpoint "GET" "/schemas" "List all schemas"

# Test new state management endpoints
test_endpoint "GET" "/schemas/available" "List available schemas"
test_endpoint "GET" "/schemas/by-state/available" "List schemas by state (available)"
test_endpoint "GET" "/schemas/by-state/approved" "List schemas by state (approved)"
test_endpoint "GET" "/schemas/by-state/blocked" "List schemas by state (blocked)"

# Test with invalid state
test_endpoint "GET" "/schemas/by-state/invalid" "List schemas by invalid state (should fail)"

# Test schema state operations (these might fail if no schemas exist)
test_endpoint "GET" "/schema/UserProfile/state" "Get UserProfile schema state"
test_endpoint "POST" "/schema/UserProfile/approve" "Approve UserProfile schema"
test_endpoint "GET" "/schema/UserProfile/state" "Get UserProfile schema state after approval"
test_endpoint "POST" "/schema/UserProfile/block" "Block UserProfile schema"
test_endpoint "GET" "/schema/UserProfile/state" "Get UserProfile schema state after blocking"

# Test with non-existent schema
test_endpoint "GET" "/schema/NonExistentSchema/state" "Get non-existent schema state (should fail)"
test_endpoint "POST" "/schema/NonExistentSchema/approve" "Approve non-existent schema (should fail)"

echo ""
echo "🎉 API endpoint testing completed!"
echo "================================================"