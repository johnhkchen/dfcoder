use crate::*;
use std::time::Duration;

/// Condition evaluation implementations
#[derive(Debug, Clone)]
pub enum ConditionType {
    /// Time-based conditions
    TimeElapsed(Duration),
    TimeBefore(chrono::DateTime<chrono::Utc>),
    TimeAfter(chrono::DateTime<chrono::Utc>),
    
    /// Agent state conditions
    AgentStatus(String, AgentStatus),
    AgentIdle(String, Duration),
    AgentTaskCount(String, u32),
    
    /// File system conditions
    FileExists(String),
    FileModified(String, Duration),
    DirectoryExists(String),
    
    /// Pane conditions
    PaneContent(u32, String),
    PaneHasErrors(u32),
    PaneActive(u32),
    
    /// Event conditions
    EventOccurred(String),
    EventCount(String, u32),
    
    /// Composite conditions
    And(Vec<ConditionType>),
    Or(Vec<ConditionType>),
    Not(Box<ConditionType>),
}

impl traits::Condition for ConditionType {
    fn evaluate(&self, context: &EvaluationContext) -> bool {
        match self {
            ConditionType::TimeElapsed(duration) => {
                // Simplified time check - in real implementation would track start time
                true
            }
            ConditionType::TimeBefore(time) => {
                context.current_time < *time
            }
            ConditionType::TimeAfter(time) => {
                context.current_time > *time
            }
            ConditionType::AgentStatus(agent_id, expected_status) => {
                context.agents.get(agent_id)
                    .map(|state| matches!(state.status, expected_status))
                    .unwrap_or(false)
            }
            ConditionType::AgentIdle(agent_id, duration) => {
                context.agents.get(agent_id)
                    .map(|state| {
                        matches!(state.status, AgentStatus::Idle) &&
                        state.last_activity.elapsed() >= *duration
                    })
                    .unwrap_or(false)
            }
            ConditionType::AgentTaskCount(agent_id, min_count) => {
                context.agents.get(agent_id)
                    .map(|state| state.tasks_completed >= *min_count)
                    .unwrap_or(false)
            }
            ConditionType::FileExists(path) => {
                std::path::Path::new(path).exists()
            }
            ConditionType::FileModified(path, duration) => {
                std::fs::metadata(path)
                    .and_then(|metadata| metadata.modified())
                    .map(|modified| modified.elapsed().unwrap_or_default() <= *duration)
                    .unwrap_or(false)
            }
            ConditionType::DirectoryExists(path) => {
                std::path::Path::new(path).is_dir()
            }
            ConditionType::PaneContent(pane_id, content) => {
                context.panes.get(pane_id)
                    .map(|pane| pane.content.contains(content))
                    .unwrap_or(false)
            }
            ConditionType::PaneHasErrors(pane_id) => {
                context.panes.get(pane_id)
                    .map(|pane| pane.has_errors)
                    .unwrap_or(false)
            }
            ConditionType::PaneActive(pane_id) => {
                context.panes.get(pane_id)
                    .map(|pane| pane.is_active)
                    .unwrap_or(false)
            }
            ConditionType::EventOccurred(event_type) => {
                context.events.iter().any(|event| {
                    match (event_type.as_str(), event) {
                        ("SupervisionRequested", SystemEvent::SupervisionRequested { .. }) => true,
                        ("TaskCompleted", SystemEvent::TaskCompleted { .. }) => true,
                        ("ErrorOccurred", SystemEvent::ErrorOccurred { .. }) => true,
                        ("AgentStateChanged", SystemEvent::AgentStateChanged { .. }) => true,
                        _ => false,
                    }
                })
            }
            ConditionType::EventCount(event_type, min_count) => {
                let count = context.events.iter().filter(|event| {
                    match (event_type.as_str(), event) {
                        ("SupervisionRequested", SystemEvent::SupervisionRequested { .. }) => true,
                        ("TaskCompleted", SystemEvent::TaskCompleted { .. }) => true,
                        ("ErrorOccurred", SystemEvent::ErrorOccurred { .. }) => true,
                        ("AgentStateChanged", SystemEvent::AgentStateChanged { .. }) => true,
                        _ => false,
                    }
                }).count();
                count >= *min_count as usize
            }
            ConditionType::And(conditions) => {
                conditions.iter().all(|c| c.evaluate(context))
            }
            ConditionType::Or(conditions) => {
                conditions.iter().any(|c| c.evaluate(context))
            }
            ConditionType::Not(condition) => {
                !condition.evaluate(context)
            }
        }
    }
    
    fn description(&self) -> String {
        match self {
            ConditionType::TimeElapsed(duration) => {
                format!("Time elapsed: {:?}", duration)
            }
            ConditionType::TimeBefore(time) => {
                format!("Before: {}", time.format("%Y-%m-%d %H:%M:%S UTC"))
            }
            ConditionType::TimeAfter(time) => {
                format!("After: {}", time.format("%Y-%m-%d %H:%M:%S UTC"))
            }
            ConditionType::AgentStatus(agent_id, status) => {
                format!("Agent '{}' has status {:?}", agent_id, status)
            }
            ConditionType::AgentIdle(agent_id, duration) => {
                format!("Agent '{}' idle for {:?}", agent_id, duration)
            }
            ConditionType::AgentTaskCount(agent_id, count) => {
                format!("Agent '{}' completed {} tasks", agent_id, count)
            }
            ConditionType::FileExists(path) => {
                format!("File exists: {}", path)
            }
            ConditionType::FileModified(path, duration) => {
                format!("File '{}' modified within {:?}", path, duration)
            }
            ConditionType::DirectoryExists(path) => {
                format!("Directory exists: {}", path)
            }
            ConditionType::PaneContent(pane_id, content) => {
                format!("Pane {} contains '{}'", pane_id, content)
            }
            ConditionType::PaneHasErrors(pane_id) => {
                format!("Pane {} has errors", pane_id)
            }
            ConditionType::PaneActive(pane_id) => {
                format!("Pane {} is active", pane_id)
            }
            ConditionType::EventOccurred(event_type) => {
                format!("Event occurred: {}", event_type)
            }
            ConditionType::EventCount(event_type, count) => {
                format!("At least {} '{}' events occurred", count, event_type)
            }
            ConditionType::And(conditions) => {
                let descriptions: Vec<String> = conditions.iter().map(|c| c.description()).collect();
                format!("All of: [{}]", descriptions.join(", "))
            }
            ConditionType::Or(conditions) => {
                let descriptions: Vec<String> = conditions.iter().map(|c| c.description()).collect();
                format!("Any of: [{}]", descriptions.join(", "))
            }
            ConditionType::Not(condition) => {
                format!("Not: {}", condition.description())
            }
        }
    }
}

/// Builder for creating complex conditions
pub struct ConditionBuilder {
    conditions: Vec<ConditionType>,
}

impl ConditionBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }
    
    /// Add a time-based condition
    pub fn time_elapsed(mut self, duration: Duration) -> Self {
        self.conditions.push(ConditionType::TimeElapsed(duration));
        self
    }
    
    /// Add an agent status condition
    pub fn agent_status(mut self, agent_id: impl Into<String>, status: AgentStatus) -> Self {
        self.conditions.push(ConditionType::AgentStatus(agent_id.into(), status));
        self
    }
    
    /// Add an agent idle condition
    pub fn agent_idle(mut self, agent_id: impl Into<String>, duration: Duration) -> Self {
        self.conditions.push(ConditionType::AgentIdle(agent_id.into(), duration));
        self
    }
    
    /// Add a file existence condition
    pub fn file_exists(mut self, path: impl Into<String>) -> Self {
        self.conditions.push(ConditionType::FileExists(path.into()));
        self
    }
    
    /// Add a pane content condition
    pub fn pane_contains(mut self, pane_id: u32, content: impl Into<String>) -> Self {
        self.conditions.push(ConditionType::PaneContent(pane_id, content.into()));
        self
    }
    
    /// Add a pane error condition
    pub fn pane_has_errors(mut self, pane_id: u32) -> Self {
        self.conditions.push(ConditionType::PaneHasErrors(pane_id));
        self
    }
    
    /// Add an event occurrence condition
    pub fn event_occurred(mut self, event_type: impl Into<String>) -> Self {
        self.conditions.push(ConditionType::EventOccurred(event_type.into()));
        self
    }
    
    /// Build an AND condition
    pub fn and(self) -> ConditionType {
        if self.conditions.len() == 1 {
            self.conditions.into_iter().next().unwrap()
        } else {
            ConditionType::And(self.conditions)
        }
    }
    
    /// Build an OR condition
    pub fn or(self) -> ConditionType {
        if self.conditions.len() == 1 {
            self.conditions.into_iter().next().unwrap()
        } else {
            ConditionType::Or(self.conditions)
        }
    }
    
    /// Build a NOT condition (only works with single condition)
    pub fn not(self) -> ConditionType {
        if self.conditions.len() == 1 {
            ConditionType::Not(Box::new(self.conditions.into_iter().next().unwrap()))
        } else {
            panic!("NOT condition builder requires exactly one condition");
        }
    }
}

impl Default for ConditionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined condition patterns for common scenarios
pub struct ConditionPatterns;

impl ConditionPatterns {
    /// Agent is stuck (idle for too long while having a task)
    pub fn agent_stuck(agent_id: &str, threshold: Duration) -> ConditionType {
        ConditionBuilder::new()
            .agent_status(agent_id, AgentStatus::Working)
            .agent_idle(agent_id, threshold)
            .and()
    }
    
    /// Build has errors
    pub fn build_has_errors(pane_id: u32) -> ConditionType {
        ConditionBuilder::new()
            .pane_contains(pane_id, "error:")
            .pane_has_errors(pane_id)
            .and()
    }
    
    /// Tests are failing
    pub fn tests_failing(pane_id: u32) -> ConditionType {
        ConditionBuilder::new()
            .pane_contains(pane_id, "FAILED")
            .pane_contains(pane_id, "test result:")
            .and()
    }
    
    /// Agent needs supervision
    pub fn agent_needs_supervision(agent_id: &str) -> ConditionType {
        ConditionBuilder::new()
            .agent_status(agent_id, AgentStatus::NeedsSupervision)
            .event_occurred("SupervisionRequested")
            .or()
    }
    
    /// High activity period (multiple events in short time)
    pub fn high_activity_period() -> ConditionType {
        ConditionBuilder::new()
            .event_occurred("AgentStateChanged")
            .event_occurred("TaskCompleted")
            .or()
    }
    
    /// Project is ready for deployment
    pub fn ready_for_deployment(build_pane: u32, test_pane: u32) -> ConditionType {
        ConditionBuilder::new()
            .pane_contains(build_pane, "Finished release")
            .pane_contains(test_pane, "test result: ok")
            .and()
    }
}

/// Condition monitor that continuously evaluates conditions
#[derive(Debug)]
pub struct ConditionMonitor {
    conditions: Vec<(String, ConditionType)>,
    evaluation_interval: Duration,
}

impl ConditionMonitor {
    pub fn new(evaluation_interval: Duration) -> Self {
        Self {
            conditions: Vec::new(),
            evaluation_interval,
        }
    }
    
    /// Add a condition to monitor
    pub fn add_condition(&mut self, name: String, condition: ConditionType) {
        self.conditions.push((name, condition));
    }
    
    /// Remove a condition by name
    pub fn remove_condition(&mut self, name: &str) -> bool {
        let original_len = self.conditions.len();
        self.conditions.retain(|(n, _)| n != name);
        self.conditions.len() != original_len
    }
    
    /// Evaluate all conditions
    pub fn evaluate_all(&self, context: &EvaluationContext) -> Vec<(String, bool)> {
        self.conditions
            .iter()
            .map(|(name, condition)| (name.clone(), condition.evaluate(context)))
            .collect()
    }
    
    /// Get conditions that are currently true
    pub fn true_conditions(&self, context: &EvaluationContext) -> Vec<String> {
        self.evaluate_all(context)
            .into_iter()
            .filter_map(|(name, result)| if result { Some(name) } else { None })
            .collect()
    }
    
    /// Start monitoring (would run in background)
    pub async fn start_monitoring<F>(&self, mut callback: F) -> Result<(), DslError>
    where
        F: FnMut(Vec<(String, bool)>) + Send + 'static,
    {
        // In a real implementation, this would start a background task
        // that periodically evaluates conditions and calls the callback
        Ok(())
    }
}

impl Default for ConditionMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}