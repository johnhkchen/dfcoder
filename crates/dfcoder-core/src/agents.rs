//! Agent system with role-based behavior

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Unique identifier for agents
pub type AgentId = String;

/// Unique identifier for tasks
pub type TaskId = String;

/// Core agent with role-based prompting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Agent {
    pub id: AgentId,
    pub role: AgentRole,
    pub pane_id: u32,
    pub current_task: Option<TaskId>,
    pub status: AgentStatus,
    #[serde(skip)]
    pub created_at: Instant,
    #[serde(skip)]
    pub last_activity: Instant,
    pub metrics: AgentMetrics,
}

/// Four clear agent roles that map to specific behaviors
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// Creates project structure and boilerplate
    Scaffolder,
    /// Writes feature code according to specifications
    Implementer,
    /// Finds and fixes bugs, improves code quality
    Debugger,
    /// Writes comprehensive test coverage
    Tester,
}

/// Current status of an agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    Stuck,
    NeedsSupervision,
    Error,
}

/// Task representation with clear requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub required_role: AgentRole,
    pub status: TaskStatus,
    #[serde(skip)]
    pub created_at: Instant,
    #[serde(skip)]
    pub assigned_at: Option<Instant>,
    #[serde(skip)]
    pub completed_at: Option<Instant>,
    pub assignee: Option<AgentId>,
    pub context: TaskContext,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Context information for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub files: Vec<String>,
    pub dependencies: Vec<TaskId>,
    pub priority: TaskPriority,
    pub estimated_duration: Option<Duration>,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Performance metrics for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    #[serde(skip)]
    pub average_completion_time: Duration,
    pub success_rate: f32,
    pub help_requests: u32,
    pub last_error: Option<String>,
}

impl Agent {
    /// Create a new agent with the specified role
    pub fn new(role: AgentRole, pane_id: u32) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            pane_id,
            current_task: None,
            status: AgentStatus::Idle,
            created_at: now,
            last_activity: now,
            metrics: AgentMetrics::default(),
        }
    }

    /// Get the system prompt for this agent's role
    pub fn system_prompt(&self) -> String {
        match self.role {
            AgentRole::Scaffolder => {
                "You are a Scaffolder agent. Your role is to create clean project structure and boilerplate code. 
                Focus on:
                - Setting up directory structures
                - Creating configuration files
                - Establishing coding conventions
                - Setting up build systems and tooling
                - Creating template files and basic structure
                
                Always prioritize clean, conventional project organization."
            }
            AgentRole::Implementer => {
                "You are an Implementer agent. Your role is to build features according to specifications.
                Focus on:
                - Writing feature implementation code
                - Following existing code patterns
                - Implementing business logic
                - Creating user interfaces
                - Integrating with external services
                
                Always write clean, maintainable code that follows project conventions."
            }
            AgentRole::Debugger => {
                "You are a Debugger agent. Your role is to find and fix bugs and improve code quality.
                Focus on:
                - Analyzing error messages and stack traces
                - Identifying root causes of issues
                - Fixing bugs with minimal changes
                - Improving code performance
                - Refactoring problematic code
                
                Always fix issues thoroughly while maintaining existing functionality."
            }
            AgentRole::Tester => {
                "You are a Tester agent. Your role is to write comprehensive test coverage.
                Focus on:
                - Writing unit tests for functions and methods
                - Creating integration tests for workflows
                - Testing edge cases and error conditions
                - Ensuring test maintainability
                - Achieving good test coverage
                
                Always write clear, reliable tests that catch regressions."
            }
        }.to_string()
    }

    /// Check if agent can handle a specific task
    pub fn can_handle_task(&self, task: &Task) -> bool {
        self.role == task.required_role && self.status == AgentStatus::Idle
    }

    /// Assign a task to this agent
    pub fn assign_task(&mut self, task_id: TaskId) -> Result<(), String> {
        if self.status != AgentStatus::Idle {
            return Err(format!("Agent {} is not idle (status: {:?})", self.id, self.status));
        }

        self.current_task = Some(task_id);
        self.status = AgentStatus::Working;
        self.last_activity = Instant::now();
        
        Ok(())
    }

    /// Complete the current task
    pub fn complete_task(&mut self) -> Result<TaskId, String> {
        let task_id = self.current_task.take()
            .ok_or("No task assigned to complete")?;

        self.status = AgentStatus::Idle;
        self.last_activity = Instant::now();
        self.metrics.tasks_completed += 1;
        
        // Update success rate
        let total_tasks = self.metrics.tasks_completed + self.metrics.tasks_failed;
        self.metrics.success_rate = self.metrics.tasks_completed as f32 / total_tasks as f32;
        
        Ok(task_id)
    }

    /// Mark the current task as failed
    pub fn fail_task(&mut self, error: String) -> Result<TaskId, String> {
        let task_id = self.current_task.take()
            .ok_or("No task assigned to fail")?;

        self.status = AgentStatus::Error;
        self.last_activity = Instant::now();
        self.metrics.tasks_failed += 1;
        self.metrics.last_error = Some(error);
        
        // Update success rate
        let total_tasks = self.metrics.tasks_completed + self.metrics.tasks_failed;
        if total_tasks > 0 {
            self.metrics.success_rate = self.metrics.tasks_completed as f32 / total_tasks as f32;
        }
        
        Ok(task_id)
    }

    /// Request help/supervision
    pub fn request_help(&mut self) {
        self.status = AgentStatus::NeedsSupervision;
        self.metrics.help_requests += 1;
        self.last_activity = Instant::now();
    }

    /// Check if agent has been idle too long
    pub fn is_idle_too_long(&self, threshold: Duration) -> bool {
        self.status == AgentStatus::Idle && self.last_activity.elapsed() > threshold
    }

    /// Check if agent has been working too long without progress
    pub fn is_stuck(&self, threshold: Duration) -> bool {
        matches!(self.status, AgentStatus::Working) && self.last_activity.elapsed() > threshold
    }

    /// Update activity timestamp
    pub fn mark_activity(&mut self) {
        self.last_activity = Instant::now();
    }
}

impl Task {
    /// Create a new task
    pub fn new(
        title: String,
        description: String,
        required_role: AgentRole,
        priority: TaskPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            required_role,
            status: TaskStatus::Pending,
            created_at: Instant::now(),
            assigned_at: None,
            completed_at: None,
            assignee: None,
            context: TaskContext {
                files: Vec::new(),
                dependencies: Vec::new(),
                priority,
                estimated_duration: None,
            },
        }
    }

    /// Assign task to an agent
    pub fn assign_to(&mut self, agent_id: AgentId) {
        self.assignee = Some(agent_id);
        self.status = TaskStatus::Assigned;
        self.assigned_at = Some(Instant::now());
    }

    /// Start task execution
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
    }

    /// Complete the task
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Instant::now());
    }

    /// Fail the task
    pub fn fail(&mut self) {
        self.status = TaskStatus::Failed;
    }

    /// Get task duration if completed
    pub fn duration(&self) -> Option<Duration> {
        match (self.assigned_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    /// Check if task dependencies are satisfied
    pub fn dependencies_satisfied(&self, completed_tasks: &[TaskId]) -> bool {
        self.context.dependencies.iter()
            .all(|dep| completed_tasks.contains(dep))
    }
}

impl Default for Agent {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            id: String::new(),
            role: AgentRole::Implementer,
            pane_id: 0,
            current_task: None,
            status: AgentStatus::Idle,
            created_at: now,
            last_activity: now,
            metrics: AgentMetrics::default(),
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            id: String::new(),
            title: String::new(),
            description: String::new(),
            required_role: AgentRole::Implementer,
            status: TaskStatus::Pending,
            created_at: now,
            assigned_at: None,
            completed_at: None,
            assignee: None,
            context: TaskContext::default(),
        }
    }
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            dependencies: Vec::new(),
            priority: TaskPriority::Normal,
            estimated_duration: None,
        }
    }
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            average_completion_time: Duration::from_secs(0),
            success_rate: 0.0,
            help_requests: 0,
            last_error: None,
        }
    }
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Scaffolder => write!(f, "Scaffolder"),
            AgentRole::Implementer => write!(f, "Implementer"),
            AgentRole::Debugger => write!(f, "Debugger"),
            AgentRole::Tester => write!(f, "Tester"),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Normal => write!(f, "Normal"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(AgentRole::Implementer, 1);
        assert_eq!(agent.role, AgentRole::Implementer);
        assert_eq!(agent.pane_id, 1);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!(agent.current_task.is_none());
    }

    #[test]
    fn test_system_prompts() {
        let scaffolder = Agent::new(AgentRole::Scaffolder, 1);
        let implementer = Agent::new(AgentRole::Implementer, 2);
        let debugger = Agent::new(AgentRole::Debugger, 3);
        let tester = Agent::new(AgentRole::Tester, 4);

        assert!(scaffolder.system_prompt().contains("Scaffolder"));
        assert!(implementer.system_prompt().contains("Implementer"));
        assert!(debugger.system_prompt().contains("Debugger"));
        assert!(tester.system_prompt().contains("Tester"));
    }

    #[test]
    fn test_task_assignment() {
        let mut agent = Agent::new(AgentRole::Implementer, 1);
        let task_id = "test-task".to_string();

        assert!(agent.assign_task(task_id.clone()).is_ok());
        assert_eq!(agent.current_task, Some(task_id));
        assert_eq!(agent.status, AgentStatus::Working);

        // Can't assign another task while working
        assert!(agent.assign_task("another-task".to_string()).is_err());
    }

    #[test]
    fn test_task_completion() {
        let mut agent = Agent::new(AgentRole::Implementer, 1);
        let task_id = "test-task".to_string();

        agent.assign_task(task_id.clone()).unwrap();
        let completed_task = agent.complete_task().unwrap();
        
        assert_eq!(completed_task, task_id);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!(agent.current_task.is_none());
        assert_eq!(agent.metrics.tasks_completed, 1);
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test task".to_string(),
            "A test task".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );

        assert_eq!(task.title, "Test task");
        assert_eq!(task.required_role, AgentRole::Implementer);
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.context.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_agent_can_handle_task() {
        let agent = Agent::new(AgentRole::Implementer, 1);
        let matching_task = Task::new(
            "Test".to_string(),
            "Test".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );
        let different_role_task = Task::new(
            "Test".to_string(),
            "Test".to_string(),
            AgentRole::Scaffolder,
            TaskPriority::Normal,
        );

        assert!(agent.can_handle_task(&matching_task));
        assert!(!agent.can_handle_task(&different_role_task));
    }
}