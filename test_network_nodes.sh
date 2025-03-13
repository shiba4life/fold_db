#!/bin/bash

# Script to test networking between multiple datafold nodes

# Function to display usage information
function show_usage {
  echo "Usage: $0 [command]"
  echo "Commands:"
  echo "  start1     - Start node 1 (port 3000)"
  echo "  start2     - Start node 2 (port 3001)"
  echo "  start3     - Start node 3 (port 3002)"
  echo "  init       - Initialize network on a node"
  echo "  connect    - Connect nodes (requires node IDs as arguments)"
  echo "  list       - List connected nodes"
  echo "  known      - List known nodes"
  echo "  discover   - Discover nodes"
  echo "  nodeid     - Get node ID"
  echo "  query      - Query a node (requires node ID as argument)"
  echo "  schemas    - List schemas on a node"
  echo "  help       - Show this help message"
}

# Check if command is provided
if [ $# -lt 1 ]; then
  show_usage
  exit 1
fi

# Process commands
case "$1" in
  init)
    if [ $# -lt 2 ]; then
      echo "Usage: $0 init <node_port>"
      echo "Example: $0 init 3000"
      exit 1
    fi
    
    echo "Initializing network on node at port $2..."
    curl -X POST http://localhost:$2/api/init_network \
      -H "Content-Type: application/json" \
      -d '{
        "enable_discovery": true,
        "discovery_port": 8090,
        "listen_port": 8091,
        "max_connections": 50,
        "connection_timeout_secs": 10,
        "announcement_interval_secs": 60
      }'
    echo
    ;;
    
  start1)
    echo "Starting node 1 on port 3000..."
    NODE_CONFIG=config/node1_config.json cargo run --bin datafold_node -- --port 3000
    ;;
    
  start2)
    echo "Starting node 2 on port 3001..."
    NODE_CONFIG=config/node2_config.json cargo run --bin datafold_node -- --port 3001
    ;;
    
  start3)
    echo "Starting node 3 on port 3002..."
    NODE_CONFIG=config/node3_config.json cargo run --bin datafold_node -- --port 3002
    ;;
    
  connect)
    if [ $# -lt 3 ]; then
      echo "Usage: $0 connect <from_node_port> <to_node_id>"
      echo "Example: $0 connect 3000 node2_id_here"
      exit 1
    fi
    
    echo "Connecting from node on port $2 to node $3..."
    curl -X POST http://localhost:$2/api/connect -H "Content-Type: application/json" -d "{\"node_id\": \"$3\"}"
    echo
    ;;
    
  list)
    if [ $# -lt 2 ]; then
      echo "Usage: $0 list <node_port>"
      echo "Example: $0 list 3000"
      exit 1
    fi
    
    echo "Listing connected nodes for node on port $2..."
    curl http://localhost:$2/api/connected_nodes
    echo
    ;;
    
  query)
    if [ $# -lt 3 ]; then
      echo "Usage: $0 query <from_node_port> <to_node_id>"
      echo "Example: $0 query 3000 node2_id_here"
      exit 1
    fi
    
    echo "Querying node $3 from node on port $2..."
    curl -X POST http://localhost:$2/api/query_node \
      -H "Content-Type: application/json" \
      -d "{
        \"node_id\": \"$3\",
        \"query\": {
          \"schema_name\": \"UserProfile\",
          \"fields\": [\"username\", \"email\"],
          \"pub_key\": \"\",
          \"trust_distance\": 1
        }
      }"
    echo
    ;;
    
  known)
    if [ $# -lt 2 ]; then
      echo "Usage: $0 known <node_port>"
      echo "Example: $0 known 3000"
      exit 1
    fi
    
    echo "Listing known nodes for node on port $2..."
    curl http://localhost:$2/api/known_nodes
    echo
    ;;
    
  discover)
    if [ $# -lt 2 ]; then
      echo "Usage: $0 discover <node_port>"
      echo "Example: $0 discover 3000"
      exit 1
    fi
    
    echo "Discovering nodes from node on port $2..."
    curl -X POST http://localhost:$2/api/discover
    echo
    ;;
    
  nodeid)
    if [ $# -lt 2 ]; then
      echo "Usage: $0 nodeid <node_port>"
      echo "Example: $0 nodeid 3000"
      exit 1
    fi
    
    echo "Getting node ID for node on port $2..."
    curl http://localhost:$2/api/node_id
    echo
    ;;
    
  schemas)
    if [ $# -lt 3 ]; then
      echo "Usage: $0 schemas <from_node_port> <to_node_id>"
      echo "Example: $0 schemas 3000 node2_id_here"
      exit 1
    fi
    
    echo "Listing schemas on node $3 from node on port $2..."
    curl -X POST http://localhost:$2/api/list_schemas \
      -H "Content-Type: application/json" \
      -d "{\"node_id\": \"$3\"}"
    echo
    ;;
    
  help)
    show_usage
    ;;
    
  *)
    echo "Unknown command: $1"
    show_usage
    exit 1
    ;;
esac

exit 0
