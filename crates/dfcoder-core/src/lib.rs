//! Core functionality for DFCoder

use dfcoder_types::*;

/// Placeholder trait for Event types
pub trait Event: Send + Sync {}

/// Placeholder error type
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),
}

/// Re-export common types
pub use dfcoder_types::*;