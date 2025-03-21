use std::io;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use libp2p::request_response::{self};
use crate::datafold_node::network::types::SchemaInfo;

/// Protocol name and version
#[derive(Debug, Clone)]
pub struct SchemaListProtocol;

impl AsRef<str> for SchemaListProtocol {
    fn as_ref(&self) -> &str {
        "/datafold/schema-list/1.0.0"
    }
}

/// Message types for the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaMessage {
    /// Request to list available schemas
    ListRequest,
    /// Response with available schemas
    ListResponse(Vec<SchemaInfo>),
}

/// Codec for serializing/deserializing schema messages
#[derive(Debug, Clone)]
pub struct SchemaCodec;

#[async_trait]
impl request_response::Codec for SchemaCodec {
    type Protocol = SchemaListProtocol;
    type Request = SchemaMessage;
    type Response = SchemaMessage;

    async fn read_request<T>(&mut self, _: &SchemaListProtocol, io: &mut T) -> io::Result<Self::Request> 
    where
        T: libp2p::futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        libp2p::futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
        
        if buf.is_empty() {
            return Ok(SchemaMessage::ListRequest);
        }
        
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(&mut self, _: &SchemaListProtocol, io: &mut T) -> io::Result<Self::Response> 
    where
        T: libp2p::futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        libp2p::futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
        
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(&mut self, _: &SchemaListProtocol, io: &mut T, req: SchemaMessage) -> io::Result<()> 
    where
        T: libp2p::futures::AsyncWrite + Unpin + Send,
    {
        let buf = match req {
            SchemaMessage::ListRequest => Vec::new(), // Empty message for ListRequest
            _ => serde_json::to_vec(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        };
        
        libp2p::futures::AsyncWriteExt::write_all(io, &buf).await?;
        Ok(())
    }

    async fn write_response<T>(&mut self, _: &SchemaListProtocol, io: &mut T, res: SchemaMessage) -> io::Result<()> 
    where
        T: libp2p::futures::AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        libp2p::futures::AsyncWriteExt::write_all(io, &buf).await?;
        Ok(())
    }
}
