# DataFold Network Visualizer

A web-based tool for managing and visualizing DataFold node networks.

## Features

- Start and stop DataFold nodes
- Initialize network connections
- Connect nodes to each other
- Execute cross-node queries
- Visualize the network topology
- Monitor node status

## Prerequisites

- Node.js (v14 or higher)
- npm
- DataFold node binary (compiled with `cargo build --bin datafold_node`)

## Installation

1. Install dependencies:

```bash
cd network_visualizer
npm install
```

## Usage

1. Start the visualizer server:

```bash
cd network_visualizer
npm start
```

2. Open your browser and navigate to:

```
http://localhost:8000
```

## Node Management

### Starting a Node

1. Select a port (e.g., 3000)
2. Choose a configuration file
3. Click "Start Node"

### Initializing Network

1. Select a running node
2. Configure discovery and listen ports
3. Click "Initialize Network"

### Connecting Nodes

1. Select a source node
2. Enter the target node ID
3. Click "Connect Nodes"

### Querying Nodes

1. Select a source node
2. Enter the target node ID
3. Specify schema and fields
4. Click "Execute Query"

## Architecture

The visualizer consists of:

1. **Backend Server**: Node.js Express server that manages DataFold node processes
2. **Frontend UI**: Web interface for visualizing and controlling nodes
3. **API**: RESTful endpoints for node operations

## API Endpoints

- `GET /api/nodes` - List all running nodes
- `POST /api/nodes/start` - Start a new node
- `POST /api/nodes/stop` - Stop a running node
- `POST /api/nodes/init-network` - Initialize network for a node
- `POST /api/nodes/connect` - Connect nodes
- `POST /api/nodes/query` - Execute a query from one node to another
- `GET /api/nodes/node-id` - Get a node's ID
- `GET /api/nodes/connected-nodes` - Get a node's connections

## Troubleshooting

- **Port Conflicts**: If a port is already in use, try a different port
- **Node Connection Issues**: Ensure both nodes have their networks initialized
- **Query Failures**: Verify that schemas are loaded on both nodes
