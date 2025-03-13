# Testing Networking of DataFold Nodes Locally

This guide explains how to test the networking capabilities of DataFold nodes in a local environment.

## Setup

1. **Create Node Configurations**

   Three configuration files have been created in the `config` directory:
   - `node1_config.json` - Configuration for Node 1 (port 3000)
   - `node2_config.json` - Configuration for Node 2 (port 3001)
   - `node3_config.json` - Configuration for Node 3 (port 3002)

   Each configuration specifies a different storage path to ensure nodes use separate databases.

2. **Helper Scripts**

   Two helper scripts are provided:
   - `test_network_nodes.sh` - For starting nodes and testing connections
   - `load_test_data.sh` - For loading test data into nodes

## Testing Process

### Step 1: Start Multiple Nodes

Start each node in a separate terminal:

```bash
# Terminal 1
./test_network_nodes.sh start1

# Terminal 2
./test_network_nodes.sh start2

# Terminal 3
./test_network_nodes.sh start3
```

### Step 2: Load Test Data and Initialize Network

After starting the nodes, load test data and initialize the network for each node:

```bash
# Terminal 4 (or any available terminal)
./load_test_data.sh 3000  # Load data into Node 1
./load_test_data.sh 3001  # Load data into Node 2
./load_test_data.sh 3002  # Load data into Node 3
```

This script will:
1. Load the UserProfile schema
2. Create a sample user profile
3. Initialize the network layer
4. Display the node ID

**Make note of these node IDs** as they are needed for connecting nodes.

### Step 3: Connect Nodes

Connect the nodes using the node IDs obtained in Step 2:

```bash
# Connect Node 1 to Node 2
./test_network_nodes.sh connect 3000 <node2_id>

# Connect Node 1 to Node 3
./test_network_nodes.sh connect 3000 <node3_id>

# Optionally connect Node 2 to Node 3
./test_network_nodes.sh connect 3001 <node3_id>
```

### Step 4: Verify Connections

Check that the nodes are connected:

```bash
# List connected nodes for Node 1
./test_network_nodes.sh list 3000

# List known nodes for Node 1
./test_network_nodes.sh known 3000
```

### Step 5: Test Cross-Node Queries

Query data from one node to another:

```bash
# List schemas available on Node 2 from Node 1
./test_network_nodes.sh schemas 3000 <node2_id>

# Query Node 2 from Node 1
./test_network_nodes.sh query 3000 <node2_id>

# Query Node 3 from Node 1
./test_network_nodes.sh query 3000 <node3_id>
```

### Step 6: Test Node Discovery

Test automatic node discovery:

```bash
# Discover nodes from Node 1
./test_network_nodes.sh discover 3000
```

## Troubleshooting

If you encounter issues:

1. **Check Node IDs**: Ensure you're using the correct node IDs when connecting nodes.
2. **Network Errors**: If you see network errors, check that the ports are available and not blocked.
3. **Schema Issues**: Make sure the schema is loaded on all nodes before attempting cross-node queries.
4. **Connection Failures**: Verify that the nodes are running and accessible.

## Advanced Testing

For more advanced testing:

1. **Custom Schemas**: Load custom schemas to test different data models.
2. **Trust Distance**: Modify the trust distance in the node configurations to test trust-based query restrictions.
3. **Network Partitioning**: Simulate network partitions by stopping and starting nodes.
4. **Schema Mapping**: Test field mapping between different schemas across nodes.

## API Reference

The following API endpoints are available on each node:

- `POST /api/schema` - Load a schema
- `POST /api/execute` - Execute a query or mutation
- `POST /api/init_network` - Initialize the network layer
- `POST /api/connect` - Connect to another node
- `POST /api/discover` - Discover nodes on the network
- `GET /api/connected_nodes` - List connected nodes
- `GET /api/known_nodes` - List known nodes
- `GET /api/node_id` - Get the node's ID
- `POST /api/query_node` - Query a remote node
- `POST /api/list_schemas` - List schemas on a remote node

## Script Reference

Several helper scripts are provided:

### test_network_nodes.sh

```bash
./test_network_nodes.sh [command]
```

Commands:
- `start1` - Start node 1 (port 3000)
- `start2` - Start node 2 (port 3001)
- `start3` - Start node 3 (port 3002)
- `init` - Initialize network on a node
- `connect` - Connect nodes
- `list` - List connected nodes
- `known` - List known nodes
- `discover` - Discover nodes
- `nodeid` - Get node ID
- `query` - Query a node
- `schemas` - List schemas on a node
- `help` - Show help message

### load_test_data.sh

```bash
./load_test_data.sh <node_port>
```

This script loads the UserProfile schema, creates a sample user profile, and initializes the network layer on the specified node.

## Network Visualizer

A web-based network visualizer is provided to make testing easier. It allows you to:

- Start and stop DataFold nodes
- Initialize network connections
- Connect nodes to each other
- Execute cross-node queries
- Visualize the network topology
- Monitor node status

### Starting the Network Visualizer

```bash
./start_network_visualizer.sh
```

This will start the visualizer server and open it in your default browser at http://localhost:8000.

### Using the Network Visualizer

1. **Start Nodes**: Use the "Node Management" panel to start nodes with different configurations
2. **Initialize Network**: Select a running node and initialize its network layer
3. **Connect Nodes**: Connect nodes to each other by specifying source node and target node ID
4. **Query Nodes**: Execute queries from one node to another
5. **Monitor Status**: View node status and connections in real-time

The visualizer provides a graphical representation of the network, making it easier to understand the connections between nodes.
