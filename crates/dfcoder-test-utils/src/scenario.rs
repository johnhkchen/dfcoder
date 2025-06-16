use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use dfcoder_core::{AgentStatus, AgentMetrics, SystemEvent};

/// Natural language scenario conditions
pub struct ScenarioCondition {
    pub description: String,
    pub predicate: Box<dyn Fn(&ScenarioContext) -> bool + Send + Sync>,
}

/// Context available during scenario execution
#[derive(Debug, Clone)]
pub struct ScenarioContext {
    pub agents: HashMap<String, AgentSnapshot>,
    pub panes: HashMap<u32, PaneSnapshot>,
    pub elapsed_time: Duration,
    pub events: Vec<SystemEvent>,
}

/// Snapshot of agent state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSnapshot {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    #[serde(skip)]
    pub last_activity: Instant,
    pub metrics: AgentMetrics,
}

/// Snapshot of pane state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PaneSnapshot {
    pub id: u32,
    pub content: String,
    #[serde(skip)]
    pub last_update: Instant,
    pub has_errors: bool,
    pub is_active: bool,
}


impl ScenarioCondition {
    /// Agent is working on a task for a specific duration
    pub fn agent_working_for(agent_name: &str, duration: Duration) -> Self {
        let agent_name = agent_name.to_string();
        Self {
            description: format!("Agent '{}' working for {:?}", agent_name, duration),
            predicate: Box::new(move |ctx| {
                ctx.agents.get(&agent_name)
                    .map(|agent| {
                        matches!(agent.status, AgentStatus::Working) &&
                        ctx.elapsed_time >= duration
                    })
                    .unwrap_or(false)
            }),
        }
    }
    
    /// No progress detected for an agent
    pub fn no_progress_detected(agent_name: &str, threshold: Duration) -> Self {
        let agent_name = agent_name.to_string();
        Self {
            description: format!("No progress from '{}' for {:?}", agent_name, threshold),
            predicate: Box::new(move |ctx| {
                ctx.agents.get(&agent_name)
                    .map(|agent| {
                        ctx.elapsed_time.saturating_sub(
                            agent.last_activity.elapsed()
                        ) >= threshold
                    })
                    .unwrap_or(false)
            }),
        }
    }
    
    /// Supervisor sees dialogue with context
    pub fn supervisor_sees_dialogue() -> Self {
        Self {
            description: "Supervisor sees dialogue with context".to_string(),
            predicate: Box::new(|ctx| {
                ctx.events.iter().any(|event| {
                    matches!(event, SystemEvent::SupervisionRequested { .. })
                })
            }),
        }
    }
    
    /// Agent status matches expected value
    pub fn agent_status(agent_name: &str, expected_status: AgentStatus) -> Self {
        let agent_name = agent_name.to_string();
        Self {
            description: format!("Agent '{}' has status {:?}", agent_name, expected_status),
            predicate: Box::new(move |ctx| {
                ctx.agents.get(&agent_name)
                    .map(|agent| agent.status == expected_status)
                    .unwrap_or(false)
            }),
        }
    }
    
    /// Pane contains specific content
    pub fn pane_contains(pane_id: u32, content: &str) -> Self {
        let content = content.to_string();
        Self {
            description: format!("Pane {} contains '{}'", pane_id, content),
            predicate: Box::new(move |ctx| {
                ctx.panes.get(&pane_id)
                    .map(|pane| pane.content.contains(&content))
                    .unwrap_or(false)
            }),
        }
    }
    
    /// Error detected in pane
    pub fn pane_has_errors(pane_id: u32) -> Self {
        Self {
            description: format!("Pane {} has errors", pane_id),
            predicate: Box::new(move |ctx| {
                ctx.panes.get(&pane_id)
                    .map(|pane| pane.has_errors)
                    .unwrap_or(false)
            }),
        }
    }
}

/// Builder for complex scenario conditions
pub struct ConditionBuilder {
    conditions: Vec<ScenarioCondition>,
}

impl ConditionBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }
    
    pub fn and(mut self, condition: ScenarioCondition) -> Self {
        self.conditions.push(condition);
        self
    }
    
    pub fn build(self) -> ScenarioCondition {
        ScenarioCondition {
            description: self.conditions.iter()
                .map(|c| &c.description)
                .collect::<Vec<_>>()
                .into_iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" AND "),
            predicate: Box::new(move |ctx| {
                self.conditions.iter().all(|c| (c.predicate)(ctx))
            }),
        }
    }
}

impl Default for ConditionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AgentSnapshot {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            status: AgentStatus::default(),
            current_task: None,
            last_activity: Instant::now(),
            metrics: AgentMetrics::default(),
        }
    }
}

impl Default for PaneSnapshot {
    fn default() -> Self {
        Self {
            id: 0,
            content: String::new(),
            last_update: Instant::now(),
            has_errors: false,
            is_active: false,
        }
    }
}

/// Convenient macros for building conditions
#[macro_export]
macro_rules! given {
    (agent $agent:ident working on $task:expr, duration $duration:expr) => {
        ScenarioCondition::agent_working_for(stringify!($agent), $duration)
    };
    (no progress detected, duration $duration:expr) => {
        ScenarioCondition::no_progress_detected("default_agent", $duration)
    };
}

#[macro_export]
macro_rules! when {
    (no progress detected) => {
        ScenarioCondition::no_progress_detected("default_agent", Duration::from_secs(0))
    };
}

#[macro_export]
macro_rules! then {
    (supervisor sees dialogue with context) => {
        ScenarioCondition::supervisor_sees_dialogue()
    };
    (agent $agent:ident has status $status:expr) => {
        ScenarioCondition::agent_status(stringify!($agent), $status)
    };
}