# LibP2P Implementation Plan and Status

## Overview

This document outlines the implementation plan and current status of the libp2p networking layer in FoldDB, including core components, integration points, and recommended best practices for a stable, decentralized system.

## Current Implementation Status

### Completed Components
1. Basic Structure
   - LibP2pNetwork class for network operations
   - LibP2pManager wrapper for compatibility
   - SchemaListProtocol for schema listing
   - FoldDbBehaviour for network behavior

2. Dependencies
   - Added core libp2p crate with necessary features
   - Included support for various protocols (noise, yamux, gossipsub, mdns, kad, etc.)

3. Basic Network Operations
   - Implemented start/stop functionality
   - Added support for node discovery
   - Added support for connecting to nodes
   - Implemented remote querying and schema listing

### In Progress Components
1. Schema Protocol
   - Created SchemaListProtocol for schema listing
   - Implemented SchemaCodec for serializing/deserializing schema messages
   - Added SchemaMessage enum for request/response messages
   - Working on fixing syntax issues and ensuring proper parameter passing

2. Network Behavior
   - Created FoldDbBehaviour for handling network events
   - Added support for mDNS discovery
   - Working on integrating with the LibP2pNetwork implementation

### Encountered Issues
1. Schema Protocol Implementation
   - Syntax errors in the SchemaCodec implementation
   - Issues with parameter passing in function signatures
   - Missing commas in generic type parameters
   - Challenges with the libp2p request-response protocol

2. Network Behavior Integration
   - Challenges with integrating mDNS discovery
   - Issues with the libp2p NetworkBehaviour trait
   - Difficulties with event handling

## DataFold System Context

### What is DataFold?

DataFold is a database system that provides:
- Schema-based data storage with atomic operations
- Fine-grained permissions control at the field level
- Trust-based access control with explicit permissions and trust distance
- Version history tracking for data changes
- Pay-per-query access using Lightning Network
- Schema transformation and interpretation

### Key System Features
1. Data Storage
   - Immutable versioning
   - Atomic operations
   - Schema validation
2. Access Control
   - Field-level permissions
   - Trust-based access
   - Public key authentication
3. Payment System
   - Lightning Network payments
   - Dynamic pricing based on trust
   - Hold invoice support
   - Payment verification
4. Schema Management
   - JSON schema definitions
   - Schema transformation
   - Field configurations
   - Validation rules

### DataFold Node

The DataFold Node:
- Manages local FoldDB instances
- Handles schema management
- Executes mutations and queries
- Enforces permissions and trust distances
- Manages atomic operations

### How LibP2P Fits In

Libp2p is the networking backbone, enabling:
1. Decentralized discovery and communication
2. Encrypted data exchange
3. Trust-based access control
4. Schema sharing and validation
5. Remote query execution with permission checks
6. Payment verification for data access

## System Integration

```
graph TD
    A[DataFoldNode] --> B[LibP2pManager]
    B --> C[LibP2pNetwork]
    C --> D[Transport Layer]
    C --> E[Discovery]
    C --> F[Peer Management]
    
    D --> G[TCP Transport]
    D --> H[WebSocket Transport]
    D --> O[QUIC (Optional)]
    
    E --> I[mDNS]
    E --> J[Kademlia DHT]
    
    F --> K[Connection Management]
    F --> L[Peer Statistics]
    
    M[Query Service] --> B
    N[Schema Service] --> B
```

## Integration Points

### 1. DataFoldNode Integration

```rust
impl DataFoldNode {
    pub fn new(config: NodeConfig) -> FoldDbResult<Self> {
        let network = LibP2pManager::new(
            config.network_config.clone(),
            config.node_id,
            config.public_key
        )?;
        
        // Set up callbacks for handling queries and schema requests
        network.set_query_callback(|query| {
            // Handle incoming queries
            self.handle_query(query)
        });
        
        network.set_schema_list_callback(|| {
            // Return available schemas
            self.list_schemas()
        });
        
        Ok(Self {
            network,
            // ... other fields
        })
    }
    
    // Core DataFold operations that integrate with networking
    pub async fn query_remote(&self, node_id: &NodeId, query: Query) -> FoldDbResult<QueryResult> {
        self.validate_trust_distance(node_id)?;
        let payment_required = self.calculate_payment(query.clone())?;
        
        // If LN payment is required, handle invoice creation & check before sending
        if payment_required > 0 {
            self.verify_lightning_payment(payment_required)?;
        }
        
        // Execute remote query through network layer
        self.network.query_node(node_id, query).await
    }
    
    pub async fn list_remote_schemas(&self, node_id: &NodeId) -> FoldDbResult<Vec<SchemaInfo>> {
        self.validate_trust_distance(node_id)?;
        self.network.list_available_schemas(node_id).await
    }
    
    // Additional LN logic (stub)
    fn verify_lightning_payment(&self, amount: u64) -> FoldDbResult<()> {
        // e.g. create invoice, check payment, etc.
        // If not paid, return Err(FoldDbError::PaymentRequired)
        Ok(())
    }
}
```

### 2. Query Service Integration

```rust
impl QueryService {
    pub async fn query_remote_node(
        &self,
        node_id: &NodeId,
        query: Query
    ) -> FoldDbResult<QueryResult> {
        self.network.query_node(node_id, query).await
    }
    
    pub async fn handle_incoming_query(
        &self,
        query: Query
    ) -> FoldDbResult<QueryResult> {
        // Validate permissions, trust distance, etc.
        self.permission_manager.validate_query(&query)?;
        
        // Execute query locally
        self.execute_query(query)
    }
}
```

### 3. Schema Service Integration

```rust
impl SchemaService {
    pub async fn list_remote_schemas(
        &self,
        node_id: &NodeId
    ) -> FoldDbResult<Vec<SchemaInfo>> {
        self.network.list_available_schemas(node_id).await
    }
    
    pub fn handle_schema_list_request(&self) -> Vec<SchemaInfo> {
        self.schema_manager.list_schemas()
    }
}
```

## Core Implementation

### 1. Transport Layer Setup

```rust
impl LibP2pNetwork {
    async fn setup_transport(&mut self) -> FoldDbResult<()> {
        // Base TCP transport + Noise + Yamux
        let tcp_base = libp2p::tcp::TokioTcpTransport::new(self.config.tcp_config)
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(self.keypair.clone())?)
            .multiplex(yamux::YamuxConfig::default())
            .boxed();
        
        // WebSocket wrapper around the same base
        let ws_transport = libp2p::websocket::WsConfig::new(tcp_base.clone())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(self.keypair.clone())?)
            .multiplex(yamux::YamuxConfig::default())
            .boxed();
        
        // (Optional) QUIC transport:
        // let quic_transport = libp2p_quic::TokioQuicTransport::new(...)?.boxed();
        
        // Combine all transports (TCP + WS [+ QUIC]) into a single transport
        let transport = libp2p::core::transport::choice(tcp_base.or_transport(ws_transport));
        
        // Build your main behavior separately:
        let behaviour = self.setup_behaviour().await?;
        
        self.swarm = SwarmBuilder::new(transport, behaviour, self.local_peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();
        
        Ok(())
    }
}
```

### 2. Discovery Implementation

```rust
#[derive(NetworkBehaviour)]
struct FoldDbBehaviour {
    mdns: mdns::Mdns,
    kad: kad::Kademlia<MemoryStore>,
    ping: ping::Ping,
    // Possibly a request/response behavior for queries
    // request_response: RequestResponse<MyProtocolCodec>,
}

impl LibP2pNetwork {
    async fn setup_behaviour(&mut self) -> FoldDbResult<FoldDbBehaviour> {
        // mDNS
        let mdns = if self.config.enable_mdns {
            mdns::Mdns::new(mdns::MdnsConfig::default()).await?
        } else {
            mdns::Mdns::new(mdns::MdnsConfig::disabled()).await?
        };
        
        // Kademlia
        let store = MemoryStore::new(self.local_peer_id);
        let mut kad = kad::Kademlia::new(self.local_peer_id, store);
        for bootstrap_addr in &self.config.bootstrap_peers {
            let peer_id = /* parse or known peer ID*/ self.local_peer_id;
            kad.add_address(&peer_id, bootstrap_addr.clone());
        }
        
        // Ping
        let ping = ping::Ping::default();
        
        Ok(FoldDbBehaviour {
            mdns,
            kad,
            ping,
        })
    }
}
```

### 3. Peer Management

```rust
struct PeerManager {
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    max_peers: usize,
    peer_stats: Arc<RwLock<HashMap<PeerId, PeerStats>>>,
}

#[derive(Clone, Debug)]
struct PeerInfo {
    node_id: NodeId,
    addresses: Vec<Multiaddr>,
    connection_status: ConnectionStatus,
    last_seen: DateTime<Utc>,
    capabilities: NodeCapabilities,
    trust_distance: u32,
}

impl PeerManager {
    async fn handle_peer_connected(&mut self, peer_id: PeerId, info: PeerInfo) -> FoldDbResult<()> {
        let mut peers = self.peers.write().await;
        
        if peers.len() >= self.max_peers {
            self.disconnect_lowest_priority_peer().await?;
        }
        // If trust_distance is higher than allowed, handle accordingly
        if info.trust_distance < self.min_allowed_trust_distance() {
            return Err(FoldDbError::TrustDistanceViolation);
        }
        
        peers.insert(peer_id, info.clone());
        drop(peers);
        
        // Initialize peer statistics
        let mut stats = self.peer_stats.write().await;
        stats.insert(peer_id, PeerStats::new());
        
        Ok(())
    }
    
    async fn disconnect_lowest_priority_peer(&mut self) -> FoldDbResult<()> {
        let peers = self.peers.read().await;
        let stats = self.peer_stats.read().await;
        
        // Find peer with lowest priority
        if let Some(peer_to_disconnect) = self.calculate_lowest_priority_peer(&peers, &stats) {
            drop(peers);
            drop(stats);
            self.disconnect_peer(peer_to_disconnect).await
        } else {
            Ok(())
        }
    }
}
```

### 4. Configuration Management

```rust
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    // Basic network settings
    pub listen_addresses: Vec<Multiaddr>,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    
    // Discovery settings
    pub enable_mdns: bool,
    pub enable_kademlia: bool,
    pub bootstrap_peers: Vec<Multiaddr>,
    
    // Transport settings
    pub tcp_config: TcpConfig,
    pub websocket_config: WsConfig,
    
    // Resource limits
    pub max_pending_connections: usize,
    pub max_peers_per_ip: usize,
    pub max_connection_attempts: u32,
    
    // DataFold-specific settings
    pub default_trust_distance: u32,
    pub payment_required: bool,
    pub min_trust_distance: u32,
    
    // Optional QUIC config
    // pub quic_config: Option<QuicConfig>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
                "/ip4/0.0.0.0/tcp/0/ws".parse().unwrap(),
            ],
            max_connections: 50,
            connection_timeout: Duration::from_secs(30),
            enable_mdns: true,
            enable_kademlia: true,
            bootstrap_peers: Vec::new(),
            tcp_config: TcpConfig::default(),
            websocket_config: WsConfig::default(),
            max_pending_connections: 10,
            max_peers_per_ip: 3,
            max_connection_attempts: 3,
            default_trust_distance: 1,
            payment_required: true,
            min_trust_distance: 0,
        }
    }
}
```

### 5. Event Handling

```rust
impl LibP2pNetwork {
    async fn handle_swarm_event(&mut self, event: SwarmEvent<FoldDbBehaviourEvent, io::Error>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {:?}", address);
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    if let Err(e) = self.handle_peer_discovery(peer_id, addr).await {
                        warn!("Failed peer discovery: {:?}", e);
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kad(kad::Event::RoutingUpdated { peer, .. })) => {
                if let Err(e) = self.update_peer_routing(peer).await {
                    warn!("Failed to update peer routing: {:?}", e);
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                if let Err(e) = self.handle_connection_established(peer_id).await {
                    warn!("Connection established error: {:?}", e);
                }
            }
            // ... Handle other events as needed ...
            _ => {}
        }
    }
}
```

## Key Features
1. Multi-Transport Support
   - TCP, WebSocket, (Optionally QUIC)
   - Noise encryption and Yamux multiplexing
2. Discovery
   - Local (mDNS)
   - Global (Kademlia DHT)
   - Optional bootstrap nodes
3. Connection Management
   - Resource limits (max peers, pending conns)
   - Peer prioritization (trust distance, success rate)
   - Automatic connection cleanup
4. Security
   - Noise protocol encryption
   - Public key authentication & trust checks
   - Payment enforcement via LN
   - Potential Relay/AutoNAT integration
5. Performance
   - Connection pooling
   - Message batching & streaming substreams
   - Optional QUIC for lower latency
6. DataFold Integration
   - Trust distance enforcement
   - Payment checks using LN
   - Schema validation hooks
   - Permission checks for queries

## Testing Strategy
1. Unit Tests
   - Peer discovery and management
   - Connection limit checks
   - Trust distance logic
   - Payment enforcement stubs
2. Integration Tests
   - Remote query execution
   - Schema listing across multiple nodes
   - Payment flows with LN (full invoice creation and settlement)
   - NAT / multi-transport fallback
3. Performance & Security Testing
   - Throughput tests under high concurrency
   - Latency tests with real-world network conditions
   - Security fuzzing for DoS and invalid messages
   - NAT traversal scenarios

## Next Steps
1. Implementation Order
   - Fix schema protocol implementation issues
   - Complete network behavior implementation
   - Implement transport layer setup
   - Add peer management
   - Integrate with Query/Schema services
   - Add payment handling with LN
   - Add security features
   - Optimize performance
2. Required Changes
   - Fix syntax errors in SchemaCodec implementation
   - Ensure proper parameter passing in function signatures
   - Add proper comma separation in generic type parameters
   - Implement NetworkBehaviour trait for FoldDbBehaviour
   - Add event handling for network events
   - Implement transport layer setup
   - Add peer management
   - Add security features
   - Add payment handling
3. Additional Considerations
   - Storage or caching of discovered peers (Kademlia persistence, if necessary)
   - Full-blown scoring system beyond naive trust distance
   - QUIC or Relay servers for better NAT handling
   - PubSub or partial streaming for real-time data updates
