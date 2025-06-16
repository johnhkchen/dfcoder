//! BAML activity classification for agent output

pub mod classifier;

pub use classifier::*;

/// Re-export for convenience
pub use classifier::{ActivityClassifier, ActivityClass, ActivityType, EmotionalState, ClassificationError};