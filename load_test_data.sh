#!/bin/bash

# Script to load test data into datafold nodes

# Function to display usage information
function show_usage {
  echo "Usage: $0 <node_port>"
  echo "Example: $0 3000"
}

# Check if port is provided
if [ $# -lt 1 ]; then
  show_usage
  exit 1
fi

NODE_PORT=$1
echo "Loading test data into node on port $NODE_PORT..."

# Load UserProfile schema
echo "Loading UserProfile schema..."
curl -X POST http://localhost:$NODE_PORT/api/schema \
  -H "Content-Type: application/json" \
  -d @src/datafold_node/examples/user_profile_schema.json
echo

# Create sample user profile
echo "Creating sample user profile..."
curl -X POST http://localhost:$NODE_PORT/api/execute \
  -H "Content-Type: application/json" \
  -H "x-public-key: test-key" \
  -d '{
    "payload": {
      "operation": "create_user",
      "content": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"create\",\"data\":{\"username\":\"user'$NODE_PORT'\",\"email\":\"user'$NODE_PORT'@example.com\",\"full_name\":\"User '$NODE_PORT'\",\"bio\":\"This is a test user on port '$NODE_PORT'\",\"age\":30,\"location\":\"Test Location '$NODE_PORT'\"}}"
    },
    "signature": "test-signature",
    "timestamp": '$(date +%s)'
  }'
echo

# Initialize network
echo "Initializing network..."
curl -X POST http://localhost:$NODE_PORT/api/init_network \
  -H "Content-Type: application/json" \
  -d '{
    "enable_discovery": true,
    "discovery_port": '$((8090 + $NODE_PORT - 3000))',
    "listen_port": '$((8091 + $NODE_PORT - 3000))',
    "max_connections": 50,
    "connection_timeout_secs": 10,
    "announcement_interval_secs": 60
  }'
echo

echo "Data loading complete for node on port $NODE_PORT"
echo "Node ID:"
curl http://localhost:$NODE_PORT/api/node_id
echo
