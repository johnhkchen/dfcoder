//! Core types for DFCoder system

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Agent status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    Stuck,
    NeedsSupervision,
    Error,
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Idle
    }
}

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Agent state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub current_task: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub tasks_completed: u32,
    pub metrics: AgentMetrics,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::default(),
            current_task: None,
            last_activity: chrono::Utc::now(),
            tasks_completed: 0,
            metrics: AgentMetrics::default(),
        }
    }
}

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u32,
    pub success_rate: f32,
    #[serde(with = "duration_serde")]
    pub average_task_duration: Duration,
    pub error_count: u32,
    pub errors_encountered: u32,
    pub help_requests: u32,
    pub response_time_ms: u64,
}

pub mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            success_rate: 0.0,
            average_task_duration: Duration::from_secs(0),
            error_count: 0,
            errors_encountered: 0,
            help_requests: 0,
            response_time_ms: 0,
        }
    }
}

/// Pane state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneState {
    pub id: u32,
    pub content: String,
    pub is_active: bool,
    pub has_errors: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Default for PaneState {
    fn default() -> Self {
        Self {
            id: 0,
            content: String::new(),
            is_active: false,
            has_errors: false,
            last_update: chrono::Utc::now(),
        }
    }
}

/// System events that can occur in DFCoder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    AgentStateChanged {
        agent_id: String,
        old_state: AgentState,
        new_state: AgentState,
    },
    SupervisionRequested {
        agent_id: String,
        message: String,
        context: String,
    },
    TaskCompleted {
        agent_id: String,
        task_id: String,
        result: TaskResult,
    },
    ErrorOccurred {
        agent_id: String,
        error_message: String,
        context: String,
    },
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success,
    Failed(String),
    Cancelled,
}

/// Agent identifier type
pub type AgentId = String;