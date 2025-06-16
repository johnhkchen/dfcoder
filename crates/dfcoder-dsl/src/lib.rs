//! Natural language DSLs for DFCoder agent behaviors
//! 
//! This crate provides expressive DSLs for defining agent behaviors,
//! event flows, and system interactions in natural language.

use dfcoder_macros::*;
use dfcoder_types::*;
use dfcoder_core::*;

pub use agents::*;
pub use events::*;
pub use behaviors::*;
pub use conditions::*;

mod agents;
mod events;
mod behaviors;
mod conditions;

// Re-export macros for convenience
pub use dfcoder_macros::{agent, events, scenario, baml_schema, mcp_resources};

/// Core traits for DSL components
pub mod traits {
    use super::*;
    
    /// Trait for agent behavior definitions
    #[async_trait::async_trait]
    pub trait AgentBehavior: Send + Sync {
        /// Handle a trigger condition
        async fn handle_trigger(&self, trigger: &TriggerCondition) -> Option<AgentAction>;
        
        /// Get the agent's current state
        fn current_state(&self) -> &AgentState;
        
        /// Update the agent's state
        fn update_state(&mut self, new_state: AgentState);
    }
    
    /// Trait for event handlers
    #[async_trait::async_trait]
    pub trait EventHandler<T: Event>: Send + Sync {
        /// Handle an event
        async fn handle(&self, event: T) -> Result<(), EventError>;
    }
    
    /// Trait for condition evaluation
    pub trait Condition: Send + Sync {
        /// Evaluate the condition
        fn evaluate(&self, context: &EvaluationContext) -> bool;
        
        /// Get a human-readable description
        fn description(&self) -> String;
    }
}

/// Common evaluation context for conditions
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub agents: std::collections::HashMap<String, AgentState>,
    pub panes: std::collections::HashMap<u32, PaneState>,
    pub current_time: chrono::DateTime<chrono::Utc>,
    pub events: Vec<SystemEvent>,
}

/// Trigger conditions for agent behaviors
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    /// Responds to specific pattern in input
    RespondsTo(String),
    /// Triggered when agent is idle
    WhenIdle,
    /// Triggered during supervision
    DuringSupervision,
    /// Custom condition
    Custom(Box<dyn traits::Condition>),
}

/// Actions that agents can take
#[derive(Debug, Clone)]
pub enum AgentAction {
    /// Provide a response
    Respond(String),
    /// Request help from supervisor
    RequestHelp(String),
    /// Execute a command
    ExecuteCommand(String),
    /// Monitor files or directories
    Monitor(String),
    /// Analyze code
    AnalyzeCode(String),
    /// No action
    None,
}

/// System events that can occur
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SystemEvent {
    /// Agent state changed
    AgentStateChanged {
        agent_id: String,
        old_state: AgentState,
        new_state: AgentState,
    },
    /// Supervision requested
    SupervisionRequested {
        agent_id: String,
        message: String,
        context: String,
    },
    /// Task completed
    TaskCompleted {
        agent_id: String,
        task_id: String,
        result: TaskResult,
    },
    /// Error occurred
    ErrorOccurred {
        agent_id: String,
        error_message: String,
        context: String,
    },
}

/// Result of a task execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TaskResult {
    Success,
    Failed(String),
    Cancelled,
}

/// Error types for DSL operations
#[derive(Debug, thiserror::Error)]
pub enum DslError {
    #[error("Invalid behavior definition: {0}")]
    InvalidBehavior(String),
    #[error("Event handling failed: {0}")]
    EventHandlingFailed(String),
    #[error("Condition evaluation failed: {0}")]
    ConditionEvaluationFailed(String),
    #[error("Agent action failed: {0}")]
    AgentActionFailed(String),
}