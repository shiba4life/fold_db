//! Field Retrieval Services
//!
//! This module provides event-driven field value retrieval services through FieldRetrievalService.
//! The service uses the message bus for event-driven communication instead of direct AtomManager access.

pub mod service;

pub use service::FieldRetrievalService;
