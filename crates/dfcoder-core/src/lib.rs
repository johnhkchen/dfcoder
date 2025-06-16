//! Core functionality for DFCoder

pub mod agents;
pub mod coordination;
pub mod supervision;

pub use agents::*;
pub use coordination::*;
pub use supervision::*;

/// Placeholder trait for Event types
pub trait Event: Send + Sync {}

/// Placeholder error type
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),
}