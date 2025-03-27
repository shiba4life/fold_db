# Active Context

## Current Task
Implementing mDNS discovery to automatically find peers on the local network.

## Recent Changes
1. Implemented mDNS Discovery:
   - Enhanced NetworkConfig with additional fields for mDNS discovery:
     - discovery_port: UDP port for discovery
     - connection_timeout: Timeout for connection attempts
     - announcement_interval: Interval for mDNS announcements
   - Implemented discover_nodes method in NetworkCore to actively scan for peers
   - Added background task for periodic mDNS announcements
   - Updated DataFoldNode to expose mDNS discovery functionality

2. Updated NodeConfig:
   - Added network_listen_address field to NodeConfig
   - Updated all tests and examples to use the new field
   - Added default value for network_listen_address

3. Enhanced Network Service Methods:
   - Split start_network into two methods:
     - start_network: Uses the address from the config
     - start_network_with_address: Uses a specific address
   - Updated all tests and examples to use the appropriate method

4. Added Node Discovery Methods:
   - Implemented discover_nodes method in DataFoldNode
   - Added get_known_nodes method to retrieve discovered peers
   - Updated tests to verify discovery functionality

5. Updated Documentation:
   - Updated network_layer.md with mDNS discovery details
   - Documented the new methods and configuration options
   - Added information about the mDNS discovery process

## Next Steps
1. Implement real mDNS discovery using libp2p-mdns
2. Add Kademlia DHT for wider peer discovery
3. Implement schema synchronization capabilities
4. Add node reputation tracking
5. Create more comprehensive tests for edge cases
6. Implement custom protocol handlers for specialized operations

## Implementation Details

### Network Layer Architecture
- Uses libp2p for P2P networking capabilities
- Automatic peer discovery using mDNS
- Periodic announcements to advertise node presence
- Active scanning for peers on the local network
- Privacy-preserving design (only shares schemas when explicitly requested)
- Efficient request-response protocol

### mDNS Discovery Process
1. Node starts up and initializes NetworkCore
2. If mDNS discovery is enabled, a background task is started for periodic announcements
3. Announcements are made at the configured interval (announcement_interval)
4. Other nodes on the local network receive these announcements and add the node to their known_peers list
5. Nodes can actively scan for peers using the discover_nodes method
6. Discovered peers are added to the known_peers list

### Schema Exchange Flow
1. Node A discovers Node B through mDNS discovery
2. Node A sends CheckSchemas request with list of schema names
3. Node B checks which schemas from the list are available
4. Node B responds with subset of available schemas
5. Node A receives and processes the response

### Security Considerations
- Noise protocol encryption for all connections
- PeerId-based authentication
- Automatic connection management
- Node validation checks
- Error handling for security violations
- Configurable connection limits and timeouts
