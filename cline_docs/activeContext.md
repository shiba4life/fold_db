# Active Context

## Current Task
Implementing cross-node request forwarding to enable communication between nodes.

## Recent Changes
1. Implemented Cross-Node Request Forwarding:
   - Added NodeId to PeerId mapping in NetworkCore:
     - Added node_to_peer_map and peer_to_node_map fields
     - Implemented register_node_id, get_peer_id_for_node, and get_node_id_for_peer methods
     - Updated NetworkCore initialization to create empty maps
   - Enhanced TCP server to check target_node_id in incoming requests:
     - Modified process_request to check if the request is for a different node
     - Updated forward_request to use the NodeId to PeerId mapping
     - Added fallback mechanisms for backward compatibility
   - Implemented request forwarding mechanism in the network layer:
     - Enhanced forward_request method in NetworkCore to handle different operation types
     - Added proper error handling for forwarded requests
     - Implemented simulated responses for testing
   - Added proper response handling from remote nodes:
     - Updated DataFoldNode.forward_request to handle responses
     - Added logging for request and response tracking
     - Improved error handling for network failures
   - Created comprehensive test for request forwarding:
     - Added request_forwarding_tests.rs with end-to-end test
     - Implemented test setup with two nodes and schema loading
     - Added verification of forwarded requests and responses

2. Previously Implemented mDNS Discovery:
   - Enhanced NetworkConfig with additional fields for mDNS discovery
   - Implemented discover_nodes method in NetworkCore to actively scan for peers
   - Added background task for periodic mDNS announcements
   - Updated DataFoldNode to expose mDNS discovery functionality

3. Updated NodeConfig:
   - Added network_listen_address field to NodeConfig
   - Updated all tests and examples to use the new field
   - Added default value for network_listen_address

## Next Steps
1. Implement real request forwarding using libp2p request-response protocol
2. Add schema synchronization capabilities
3. Implement Kademlia DHT for wider peer discovery
4. Add node reputation tracking
5. Create more comprehensive tests for edge cases
6. Implement custom protocol handlers for specialized operations

## Implementation Details

### Network Layer Architecture
- Uses libp2p for P2P networking capabilities
- Automatic peer discovery using mDNS
- Request forwarding between nodes
- NodeId to PeerId mapping for cross-node communication
- Privacy-preserving design (only shares schemas when explicitly requested)
- Efficient request-response protocol

### Request Forwarding Process
1. Client sends a request to Node A with a target_node_id field
2. Node A checks if the target_node_id matches its own node ID
3. If not, Node A looks up the PeerId for the target_node_id
4. Node A forwards the request to the target node using the network layer
5. Target node processes the request and sends a response
6. Node A receives the response and forwards it back to the client

### NodeId to PeerId Mapping
1. Each node has a unique NodeId (UUID) and PeerId (libp2p peer ID)
2. When a node initializes its network layer, it registers its NodeId with its PeerId
3. When nodes discover each other, they exchange NodeIds and PeerIds
4. The mapping is stored in node_to_peer_map and peer_to_node_map
5. When forwarding a request, the system looks up the PeerId for the target NodeId
6. If the mapping doesn't exist, fallback mechanisms are used

### mDNS Discovery Process
1. Node starts up and initializes NetworkCore
2. If mDNS discovery is enabled, a background task is started for periodic announcements
3. Announcements are made at the configured interval (announcement_interval)
4. Other nodes on the local network receive these announcements and add the node to their known_peers list
5. Nodes can actively scan for peers using the discover_nodes method
6. Discovered peers are added to the known_peers list

### Security Considerations
- Noise protocol encryption for all connections
- PeerId-based authentication
- Automatic connection management
- Node validation checks
- Error handling for security violations
- Configurable connection limits and timeouts
