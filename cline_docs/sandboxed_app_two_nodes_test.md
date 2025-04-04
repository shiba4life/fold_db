# Sandboxed App Two Nodes Test Documentation

## Overview

The `sandboxed_app_two_nodes` test demonstrates the DataFold system's capability to run applications in a sandboxed environment while enabling secure cross-node communication. This test validates several key features of the DataFold stack, including node setup, schema creation, data management, sandboxed application execution, and cross-node querying.

## Test Architecture

The test creates a complete end-to-end demonstration of DataFold's capabilities with the following components:

1. **Two DataFold Nodes**: Independent nodes with their own storage, network services, and TCP servers
2. **FoldClient Instances**: Mediators that provide sandboxed access to the DataFold node API
3. **Sandboxed Applications**: Applications that run in a restricted environment and communicate with DataFold nodes through FoldClient

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Sandboxed App  │◄────┤   FoldClient    │◄────┤  DataFold Node  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Test Workflow

The test follows a structured workflow to validate the entire system:

1. **Node Setup**: Creates two DataFold nodes with network connectivity
2. **Schema Creation**: Defines and creates user and post schemas on both nodes
3. **Data Population**: Adds test users and posts to each node
4. **FoldClient Setup**: Creates and configures FoldClient instances for each node
5. **Sandboxed App Execution**: Launches applications in a sandboxed environment
6. **Cross-Node Querying**: Verifies that data can be queried across nodes
7. **Cleanup**: Stops all services and cleans up resources

## Key Components Tested

### 1. DataFold Node Configuration and Setup

The test demonstrates how to configure and initialize DataFold nodes with:
- Custom storage paths
- Network configuration with specific ports
- TCP server setup for client communication
- Trust relationships between nodes
- Peer discovery and registration

### 2. Schema Management

The test showcases DataFold's schema system capabilities:
- JSON-based schema definitions
- Field-level permission policies
- Payment configuration for data access
- Schema creation through the client API

### 3. Data Operations

The test validates core data operations:
- Creating user and post records
- Storing data on different nodes
- Unique ID generation for records
- Data mutation through the client API

### 4. Sandboxed Application Environment

The test demonstrates FoldClient's sandboxing capabilities:
- Resource limits (memory, CPU)
- Network access controls
- Filesystem access controls
- Application registration and permission management
- Secure IPC communication

### 5. Cross-Node Communication

The test validates DataFold's network layer:
- Node discovery using the network layer
- NodeId to PeerId mapping
- Request forwarding between nodes
- Cross-node querying
- Result verification

## DataFold Stack Insights

From this test, we can infer several key aspects of the DataFold stack:

### Architecture

- **Modular Design**: Clear separation between node, client, and application layers
- **P2P Network Layer**: Uses libp2p for peer-to-peer communication
- **Sandboxed Execution**: Applications run in isolated environments with controlled access
- **IPC Communication**: Secure communication between applications and FoldClient

### Core Components

1. **DataFoldNode**: The main server component that:
   - Manages data storage and schemas
   - Provides network services for peer discovery and communication
   - Exposes a TCP server for client connections
   - Handles request forwarding between nodes

2. **FoldClient**: A mediator that:
   - Creates sandboxed environments for applications
   - Enforces resource limits and access controls
   - Provides secure IPC for application communication
   - Manages application lifecycle
   - Acts as a security layer between applications and DataFold nodes

3. **DataFold SDK**: A software development kit that includes:
   - `DataFoldSDKClient` class for connecting to DataFold nodes via TCP
   - APIs for schema and data operations
   - Support for cross-node querying
   - Node discovery functionality
   - Direct communication with DataFold nodes

### Relationship Between DataFold SDK and FoldClient

The test uses two separate client paths to interact with DataFold nodes:

1. **DataFold SDK's DataFoldSDKClient**: A direct client for DataFold nodes:
   - Used for system-level operations (creating schemas, adding test data)
   - Provides a programmatic API for direct node communication
   - No sandboxing or security restrictions
   - Used in the test for setup and verification purposes
   - Communicates directly with the DataFold node's TCP server

2. **FoldClient**: A security-focused mediator:
   - Not dependent on the DataFold SDK
   - Creates a secure environment for third-party applications
   - Manages application lifecycle (start, stop, monitor)
   - Enforces resource limits and access controls
   - Provides a secure IPC channel for applications
   - Acts as a security gateway between applications and DataFold nodes

These are parallel paths to DataFold nodes, not a chain where one goes through the other:

```
DataFold SDK's DataFoldSDKClient ---> DataFold Node
                                      ^
                                      |
FoldClient ---> Sandboxed App --------|
```

The SDK could potentially be eliminated for end users who only need the sandboxed application environment, but it serves important purposes:
- Provides a development interface for system components
- Enables testing and administrative operations
- Offers a simpler API for trusted applications
- Serves as a reference implementation for node communication

### Data Model

- **Schema-Based**: All data is structured according to defined schemas
- **Field-Level Permissions**: Access control at the field level
- **Payment Integration**: Support for micropayments for data access
- **Trust-Based Access**: Permission policies based on trust distance

### Network Layer

- **P2P Communication**: Nodes communicate directly with each other
- **Node Discovery**: Automatic discovery of peers on the network
- **Request Forwarding**: Ability to forward requests to other nodes
- **NodeId Mapping**: Mapping between NodeId and PeerId for routing

### Security Model

- **Sandboxed Execution**: Applications run in restricted environments
- **Resource Limits**: Controls on memory, CPU, network, and filesystem access
- **Permission Enforcement**: Explicit permissions for operations
- **Cryptographic Authentication**: All communication is authenticated

## Technical Implementation Details

### Node Setup

The test creates two nodes with different configurations:
- Each node has its own storage directory
- Network services run on different ports (9001 and 9002)
- TCP servers run on different ports (8001 and 8002)
- Nodes are configured to trust each other
- Peer IDs are manually registered to simulate discovery

### Schema Definition

The test defines two schemas:

1. **User Schema**:
   - Fields: id, username, full_name, bio, email
   - All fields have open read access but restricted write access
   - Payment configuration with base multipliers

2. **Post Schema**:
   - Fields: id, title, content, author_id
   - Similar permission and payment configuration

### FoldClient Configuration

The test configures FoldClient instances with:
- Memory limits (512MB)
- CPU limits (25%)
- Network access enabled
- Filesystem access enabled
- Connection to specific node TCP addresses

### Sandboxed App Execution

The test:
- Builds the sandboxed_app example
- Launches it with specific arguments
- Monitors its execution with timeouts
- Terminates it if necessary

### Cross-Node Querying

The test verifies cross-node functionality by:
- Discovering remote nodes
- Querying posts on Node 2 through Node 1
- Verifying that posts from Node 2 are accessible

## Conclusion

The `sandboxed_app_two_nodes` test provides a comprehensive demonstration of DataFold's capabilities, particularly focusing on sandboxed application execution and cross-node communication. It validates the system's ability to:

1. Run applications in secure, isolated environments
2. Enable controlled access to node APIs
3. Facilitate communication between nodes
4. Provide schema-based data storage and access
5. Enforce permissions and resource limits

This test serves as both a validation of the system's functionality and a reference implementation for developers building applications on the DataFold stack.
