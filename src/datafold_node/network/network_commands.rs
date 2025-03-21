use std::collections::HashSet;
use tokio::sync::mpsc;

use crate::error::FoldDbResult;
use crate::datafold_node::network::types::{NodeId, NodeInfo, SchemaInfo, QueryResult as FoldDbQueryResult};
use crate::schema::types::Query;

/// Commands that can be sent to the network task
pub enum NetworkCommand {
    /// Discover nodes on the network
    DiscoverNodes(mpsc::Sender<FoldDbResult<Vec<NodeInfo>>>),
    /// Connect to a node
    ConnectToNode(NodeId, mpsc::Sender<FoldDbResult<()>>),
    /// Query a node
    QueryNode(NodeId, Query, mpsc::Sender<FoldDbResult<FoldDbQueryResult>>),
    /// List available schemas on a node
    ListSchemas(NodeId, mpsc::Sender<FoldDbResult<Vec<SchemaInfo>>>),
    /// Get connected nodes
    GetConnectedNodes(mpsc::Sender<FoldDbResult<HashSet<NodeId>>>),
    /// Get known nodes
    GetKnownNodes(mpsc::Sender<FoldDbResult<Vec<NodeInfo>>>),
    /// Stop the network
    Stop(mpsc::Sender<FoldDbResult<()>>),
}
