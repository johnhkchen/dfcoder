//! Workshop capacity management and task coordination

use crate::agents::*;
use crate::retry::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Manages workshop capacity and task assignment
#[derive(Debug)]
pub struct WorkshopManager {
    /// Maximum concurrent agents per role
    max_concurrent: HashMap<AgentRole, usize>,
    /// Currently active agents per role
    active_agents: HashMap<AgentRole, Vec<AgentId>>,
    /// All registered agents
    agents: HashMap<AgentId, Agent>,
    /// Task queue organized by priority
    task_queue: VecDeque<Task>,
    /// Completed tasks for dependency checking
    completed_tasks: Vec<TaskId>,
    /// Workshop metrics
    metrics: WorkshopMetrics,
    /// Retry executor for failed tasks
    retry_executor: RetryExecutor,
    /// Agent expertise tracking
    agent_expertise: HashMap<AgentId, AgentExpertise>,
}

/// Tracks agent expertise and performance patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExpertise {
    /// Success rate for different task types
    pub task_success_rates: HashMap<String, f32>,
    /// Average completion time by task complexity
    pub completion_times: HashMap<TaskComplexity, Duration>,
    /// Total tasks completed
    pub total_tasks: u32,
    /// Specialization score (0.0 = generalist, 1.0 = highly specialized)
    pub specialization_score: f32,
    /// Last performance update
    #[serde(skip, default = "instant_now")]
    pub last_updated: Instant,
}

fn instant_now() -> Instant {
    Instant::now()
}

/// Task complexity levels for better assignment
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,     // < 1 hour estimated
    Medium,     // 1-4 hours estimated  
    Complex,    // 4-8 hours estimated
    Expert,     // > 8 hours estimated
}

/// Workshop performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopMetrics {
    pub total_tasks_processed: u32,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub tasks_retried: u32,
    #[serde(skip)]
    pub average_task_duration: Duration,
    pub agent_utilization: HashMap<AgentRole, f32>,
    pub queue_length: usize,
    pub bottleneck_role: Option<AgentRole>,
    /// System throughput (tasks per hour)
    pub throughput: f32,
    /// Success rate percentage
    pub success_rate: f32,
    /// Cost efficiency metrics
    pub cost_per_task: f32,
}

/// Errors that can occur in workshop management
#[derive(Debug, Error)]
pub enum WorkshopError {
    #[error("Agent {0} not found")]
    AgentNotFound(AgentId),
    #[error("Task {0} not found")]
    TaskNotFound(TaskId),
    #[error("No available agents for role {0}")]
    NoAvailableAgents(AgentRole),
    #[error("Agent {0} is already working on task {1}")]
    AgentBusy(AgentId, TaskId),
    #[error("Task dependencies not satisfied: {0:?}")]
    DependenciesNotSatisfied(Vec<TaskId>),
    #[error("Workshop at capacity for role {0}")]
    AtCapacity(AgentRole),
}

impl WorkshopManager {
    /// Create a new workshop manager with default capacity limits
    pub fn new() -> Self {
        let mut max_concurrent = HashMap::new();
        max_concurrent.insert(AgentRole::Scaffolder, 1);    // Only one scaffolder at a time
        max_concurrent.insert(AgentRole::Implementer, 3);   // Multiple implementers can work in parallel
        max_concurrent.insert(AgentRole::Debugger, 2);      // A couple debuggers can work simultaneously
        max_concurrent.insert(AgentRole::Tester, 2);        // Testers can work in parallel

        Self {
            max_concurrent,
            active_agents: HashMap::new(),
            agents: HashMap::new(),
            task_queue: VecDeque::new(),
            completed_tasks: Vec::new(),
            metrics: WorkshopMetrics::default(),
            retry_executor: RetryExecutor::new(RetryPolicy::default()),
            agent_expertise: HashMap::new(),
        }
    }

    /// Register a new agent in the workshop
    pub fn register_agent(&mut self, agent: Agent) -> Result<(), WorkshopError> {
        let agent_id = agent.id.clone();
        let role = agent.role.clone();
        
        self.agents.insert(agent_id.clone(), agent);
        
        // Initialize role tracking if needed
        if !self.active_agents.contains_key(&role) {
            self.active_agents.insert(role, Vec::new());
        }
        
        Ok(())
    }

    /// Check if we can assign a task to an agent with the given role
    pub fn can_assign(&self, role: AgentRole) -> bool {
        let active = self.active_agents.get(&role).map(|v| v.len()).unwrap_or(0);
        let max = self.max_concurrent.get(&role).copied().unwrap_or(1);
        active < max
    }

    /// Add a task to the queue
    pub fn queue_task(&mut self, task: Task) {
        // Insert task in priority order
        let insert_pos = self.task_queue.iter().position(|t| {
            t.context.priority < task.context.priority
        }).unwrap_or(self.task_queue.len());
        
        self.task_queue.insert(insert_pos, task);
        self.metrics.queue_length = self.task_queue.len();
    }

    /// Try to assign the next available task
    pub fn try_assign_next_task(&mut self) -> Result<Option<(AgentId, TaskId)>, WorkshopError> {
        // Find a task that can be assigned
        let task_index = self.find_assignable_task_index()?;
        
        if let Some(index) = task_index {
            let task = self.task_queue.remove(index).unwrap();
            let agent_id = self.assign_task(task)?;
            return Ok(Some((agent_id.clone(), agent_id)));
        }
        
        Ok(None)
    }

    /// Assign tasks by priority, considering agent expertise and load balancing
    pub fn assign_by_priority(&mut self) -> Result<Option<(AgentId, Task)>, WorkshopError> {
        // Sort tasks by priority and complexity
        self.prioritize_task_queue();
        
        // Find the best task-agent match
        for (task_index, task) in self.task_queue.iter().enumerate() {
            if !task.dependencies_satisfied(&self.completed_tasks) {
                continue;
            }
            
            if let Some(_best_agent) = self.find_best_agent_for_task(task) {
                let task = self.task_queue.remove(task_index).unwrap();
                let task_copy = task.clone();
                let agent_id = self.assign_task(task)?;
                return Ok(Some((agent_id, task_copy)));
            }
        }
        
        Ok(None)
    }

    /// Execute a task with retry logic
    pub async fn execute_task_with_retry(
        &mut self,
        agent_id: &AgentId,
        task: &Task,
    ) -> Result<TaskResult, WorkshopError> {
        let agent = self.agents.get_mut(agent_id)
            .ok_or_else(|| WorkshopError::AgentNotFound(agent_id.clone()))?;
        
        let result = self.retry_executor.execute_task(agent, task).await
            .map_err(|e| WorkshopError::TaskNotFound(format!("Retry failed: {:?}", e)))?;
        
        // Update metrics
        self.metrics.tasks_retried += result.attempt_number - 1;
        self.update_agent_expertise(agent_id, task, &result);
        
        Ok(result)
    }

    /// Find an assignable task considering dependencies and available agents
    fn find_assignable_task_index(&self) -> Result<Option<usize>, WorkshopError> {
        for (index, task) in self.task_queue.iter().enumerate() {
            // Check if dependencies are satisfied
            if !task.dependencies_satisfied(&self.completed_tasks) {
                continue;
            }
            
            // Check if we have capacity for this role
            if !self.can_assign(task.required_role.clone()) {
                continue;
            }
            
            // Check if we have an available agent
            if self.find_available_agent(task.required_role.clone()).is_some() {
                return Ok(Some(index));
            }
        }
        
        Ok(None)
    }

    /// Assign a task to an available agent
    pub fn assign_task(&mut self, mut task: Task) -> Result<AgentId, WorkshopError> {
        // Find available agent
        let agent_id = self.find_available_agent(task.required_role.clone())
            .ok_or_else(|| WorkshopError::NoAvailableAgents(task.required_role.clone()))?;

        // Get mutable reference to agent
        let agent = self.agents.get_mut(&agent_id)
            .ok_or_else(|| WorkshopError::AgentNotFound(agent_id.clone()))?;

        // Assign task to agent
        agent.assign_task(task.id.clone()).map_err(|_| {
            WorkshopError::AgentBusy(agent_id.clone(), task.id.clone())
        })?;

        // Update task
        task.assign_to(agent_id.clone());
        task.start();

        // Track active agent
        self.active_agents.entry(task.required_role.clone())
            .or_insert_with(Vec::new)
            .push(agent_id.clone());

        self.metrics.queue_length = self.task_queue.len();
        
        Ok(agent_id)
    }

    /// Find an available agent for the given role
    fn find_available_agent(&self, role: AgentRole) -> Option<AgentId> {
        self.agents.values()
            .find(|agent| agent.role == role && agent.status == AgentStatus::Idle)
            .map(|agent| agent.id.clone())
    }

    /// Mark a task as completed
    pub fn complete_task(&mut self, agent_id: AgentId, task_id: TaskId) -> Result<(), WorkshopError> {
        // Get agent and complete task
        let agent = self.agents.get_mut(&agent_id)
            .ok_or_else(|| WorkshopError::AgentNotFound(agent_id.clone()))?;

        let completed_task_id = agent.complete_task()
            .map_err(|e| WorkshopError::TaskNotFound(e))?;

        if completed_task_id != task_id {
            return Err(WorkshopError::TaskNotFound(task_id));
        }

        // Remove from active agents
        if let Some(active) = self.active_agents.get_mut(&agent.role) {
            active.retain(|id| id != &agent_id);
        }

        // Track completion
        self.completed_tasks.push(task_id);
        self.metrics.tasks_completed += 1;
        self.metrics.total_tasks_processed += 1;

        // Update success rates
        self.update_metrics();

        Ok(())
    }

    /// Mark a task as failed
    pub fn fail_task(&mut self, agent_id: AgentId, task_id: TaskId, error: String) -> Result<(), WorkshopError> {
        // Get agent and fail task
        let agent = self.agents.get_mut(&agent_id)
            .ok_or_else(|| WorkshopError::AgentNotFound(agent_id.clone()))?;

        let failed_task_id = agent.fail_task(error)
            .map_err(|e| WorkshopError::TaskNotFound(e))?;

        if failed_task_id != task_id {
            return Err(WorkshopError::TaskNotFound(task_id));
        }

        // Remove from active agents
        if let Some(active) = self.active_agents.get_mut(&agent.role) {
            active.retain(|id| id != &agent_id);
        }

        // Track failure
        self.metrics.tasks_failed += 1;
        self.metrics.total_tasks_processed += 1;

        // Update metrics
        self.update_metrics();

        Ok(())
    }

    /// Get current workshop status
    pub fn get_status(&mut self) -> WorkshopStatus {
        // Update metrics before returning status
        self.update_metrics();
        
        WorkshopStatus {
            total_agents: self.agents.len(),
            active_agents: self.active_agents.values().map(|v| v.len()).sum(),
            queue_length: self.task_queue.len(),
            capacity_per_role: self.max_concurrent.clone(),
            active_per_role: self.active_agents.iter()
                .map(|(role, agents)| (role.clone(), agents.len()))
                .collect(),
            metrics: self.metrics.clone(),
        }
    }

    /// Update workshop metrics
    fn update_metrics(&mut self) {
        // Calculate agent utilization per role
        for (role, max_capacity) in &self.max_concurrent {
            let active_count = self.active_agents.get(role).map(|v| v.len()).unwrap_or(0);
            let utilization = if *max_capacity > 0 {
                active_count as f32 / *max_capacity as f32
            } else {
                0.0
            };
            self.metrics.agent_utilization.insert(role.clone(), utilization);
        }

        // Find bottleneck role (highest utilization)
        self.metrics.bottleneck_role = self.metrics.agent_utilization.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(role, _)| role.clone());

        self.metrics.queue_length = self.task_queue.len();
    }

    /// Get agent by ID
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<&Agent> {
        self.agents.get(agent_id)
    }

    /// Get mutable agent by ID
    pub fn get_agent_mut(&mut self, agent_id: &AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(agent_id)
    }

    /// Get all agents
    pub fn get_all_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    /// Get tasks in queue
    pub fn get_queue(&self) -> &VecDeque<Task> {
        &self.task_queue
    }

    /// Check for stuck agents and request supervision
    pub fn check_for_stuck_agents(&mut self, stuck_threshold: Duration) -> Vec<AgentId> {
        let mut stuck_agents = Vec::new();
        
        for agent in self.agents.values_mut() {
            if agent.is_stuck(stuck_threshold) {
                agent.request_help();
                stuck_agents.push(agent.id.clone());
            }
        }
        
        stuck_agents
    }

    /// Set capacity for a role
    pub fn set_capacity(&mut self, role: AgentRole, capacity: usize) {
        self.max_concurrent.insert(role, capacity);
    }

    /// Prioritize task queue by priority and complexity
    fn prioritize_task_queue(&mut self) {
        // Collect tasks and sort them
        let mut tasks: Vec<_> = self.task_queue.drain(..).collect();
        tasks.sort_by(|a, b| {
            // First sort by priority
            match b.context.priority.cmp(&a.context.priority) {
                std::cmp::Ordering::Equal => {
                    // Then by complexity (simpler tasks first for quick wins)
                    let a_complexity = self.estimate_task_complexity(a);
                    let b_complexity = self.estimate_task_complexity(b);
                    a_complexity.cmp(&b_complexity)
                }
                other => other,
            }
        });
        
        // Put sorted tasks back
        for task in tasks {
            self.task_queue.push_back(task);
        }
    }

    /// Estimate task complexity based on description and requirements
    fn estimate_task_complexity(&self, task: &Task) -> TaskComplexity {
        let description = task.description.to_lowercase();
        let title = task.title.to_lowercase();
        
        // Simple heuristics for complexity estimation
        if description.contains("fix") || description.contains("bug") || title.len() < 20 {
            TaskComplexity::Simple
        } else if description.contains("implement") || description.contains("feature") {
            if description.contains("complex") || description.contains("integration") {
                TaskComplexity::Complex
            } else {
                TaskComplexity::Medium
            }
        } else if description.contains("architecture") || description.contains("design") {
            TaskComplexity::Expert
        } else {
            TaskComplexity::Medium
        }
    }

    /// Find the best agent for a given task based on expertise and availability
    fn find_best_agent_for_task(&self, task: &Task) -> Option<AgentId> {
        let available_agents: Vec<_> = self.agents.values()
            .filter(|agent| {
                agent.role == task.required_role 
                && agent.status == AgentStatus::Idle
                && self.can_assign(agent.role.clone())
            })
            .collect();

        if available_agents.is_empty() {
            return None;
        }

        // Score agents based on expertise
        let mut best_agent = None;
        let mut best_score = -1.0f32;

        for agent in available_agents {
            let score = self.calculate_agent_score(agent, task);
            if score > best_score {
                best_score = score;
                best_agent = Some(agent.id.clone());
            }
        }

        best_agent
    }

    /// Calculate agent suitability score for a task
    fn calculate_agent_score(&self, agent: &Agent, task: &Task) -> f32 {
        let expertise = self.agent_expertise.get(&agent.id);
        
        let mut score = 0.5; // Base score
        
        if let Some(exp) = expertise {
            // Factor in success rate for similar tasks
            let task_type = self.categorize_task(task);
            if let Some(success_rate) = exp.task_success_rates.get(&task_type) {
                score += success_rate * 0.3;
            }
            
            // Factor in completion time efficiency
            let complexity = self.estimate_task_complexity(task);
            if let Some(avg_time) = exp.completion_times.get(&complexity) {
                // Lower time = higher score
                let efficiency = 1.0 / (avg_time.as_secs_f32() / 3600.0 + 1.0);
                score += efficiency * 0.2;
            }
            
            // Factor in total experience
            score += (exp.total_tasks as f32 / 100.0).min(0.2);
        }
        
        // Factor in current workload (prefer less busy agents)
        score -= agent.metrics.tasks_completed as f32 * 0.01;
        
        score
    }

    /// Categorize task for expertise tracking
    fn categorize_task(&self, task: &Task) -> String {
        let description = task.description.to_lowercase();
        
        if description.contains("test") || description.contains("spec") {
            "testing".to_string()
        } else if description.contains("debug") || description.contains("fix") {
            "debugging".to_string()
        } else if description.contains("setup") || description.contains("scaffold") {
            "scaffolding".to_string()
        } else if description.contains("implement") || description.contains("feature") {
            "implementation".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Update agent expertise based on task completion
    fn update_agent_expertise(&mut self, agent_id: &AgentId, task: &Task, result: &TaskResult) {
        let task_type = self.categorize_task(task);
        let complexity = self.estimate_task_complexity(task);
        
        // Get or create expertise entry
        let expertise = self.agent_expertise.entry(agent_id.clone())
            .or_insert_with(AgentExpertise::default);
        
        // Update success rate
        let current_rate = expertise.task_success_rates.get(&task_type).copied().unwrap_or(0.5);
        let new_rate = if result.success {
            (current_rate * 0.9) + (1.0 * 0.1) // Weighted average favoring recent performance
        } else {
            (current_rate * 0.9) + (0.0 * 0.1)
        };
        expertise.task_success_rates.insert(task_type, new_rate);
        
        // Update completion time
        if result.success {
            let current_time = expertise.completion_times.get(&complexity)
                .copied().unwrap_or(Duration::from_secs(3600));
            let new_time = Duration::from_secs(
                ((current_time.as_secs() as f32 * 0.8) + (result.duration.as_secs() as f32 * 0.2)) as u64
            );
            expertise.completion_times.insert(complexity, new_time);
        }
        
        // Update total tasks and last updated
        expertise.total_tasks += 1;
        expertise.last_updated = Instant::now();
        
        // Calculate specialization score separately to avoid borrow conflicts
        let rates: Vec<f32> = expertise.task_success_rates.values().copied().collect();
        let specialization_score = if rates.is_empty() {
            0.0
        } else {
            let avg_rate = rates.iter().sum::<f32>() / rates.len() as f32;
            let variance = rates.iter()
                .map(|rate| (rate - avg_rate).powi(2))
                .sum::<f32>() / rates.len() as f32;
            variance.sqrt().min(1.0)
        };
        expertise.specialization_score = specialization_score;
    }

    /// Calculate agent specialization score
    fn calculate_specialization_score(&self, expertise: &AgentExpertise) -> f32 {
        if expertise.task_success_rates.is_empty() {
            return 0.0;
        }
        
        let rates: Vec<f32> = expertise.task_success_rates.values().copied().collect();
        let avg_rate = rates.iter().sum::<f32>() / rates.len() as f32;
        let variance = rates.iter()
            .map(|rate| (rate - avg_rate).powi(2))
            .sum::<f32>() / rates.len() as f32;
        
        // Higher variance = more specialized
        variance.sqrt().min(1.0)
    }
}

/// Current status of the workshop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopStatus {
    pub total_agents: usize,
    pub active_agents: usize,
    pub queue_length: usize,
    pub capacity_per_role: HashMap<AgentRole, usize>,
    pub active_per_role: HashMap<AgentRole, usize>,
    pub metrics: WorkshopMetrics,
}

impl Default for WorkshopMetrics {
    fn default() -> Self {
        Self {
            total_tasks_processed: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            tasks_retried: 0,
            average_task_duration: Duration::from_secs(0),
            agent_utilization: HashMap::new(),
            queue_length: 0,
            bottleneck_role: None,
            throughput: 0.0,
            success_rate: 0.0,
            cost_per_task: 0.0,
        }
    }
}

impl Default for AgentExpertise {
    fn default() -> Self {
        Self {
            task_success_rates: HashMap::new(),
            completion_times: HashMap::new(),
            total_tasks: 0,
            specialization_score: 0.0,
            last_updated: Instant::now(),
        }
    }
}

impl Default for WorkshopManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workshop_creation() {
        let workshop = WorkshopManager::new();
        assert!(workshop.can_assign(AgentRole::Implementer));
        assert!(workshop.can_assign(AgentRole::Scaffolder));
    }

    #[test]
    fn test_agent_registration() {
        let mut workshop = WorkshopManager::new();
        let agent = Agent::new(AgentRole::Implementer, 1);
        let agent_id = agent.id.clone();

        assert!(workshop.register_agent(agent).is_ok());
        assert!(workshop.get_agent(&agent_id).is_some());
    }

    #[test]
    fn test_task_queuing() {
        let mut workshop = WorkshopManager::new();
        let task = Task::new(
            "Test task".to_string(),
            "Description".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );

        workshop.queue_task(task);
        assert_eq!(workshop.get_queue().len(), 1);
    }

    #[test]
    fn test_task_assignment() {
        let mut workshop = WorkshopManager::new();
        let agent = Agent::new(AgentRole::Implementer, 1);
        let agent_id = agent.id.clone();
        workshop.register_agent(agent).unwrap();

        let task = Task::new(
            "Test task".to_string(),
            "Description".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );
        let task_id = task.id.clone();

        let assigned_agent = workshop.assign_task(task).unwrap();
        assert_eq!(assigned_agent, agent_id);

        let agent = workshop.get_agent(&agent_id).unwrap();
        assert_eq!(agent.current_task, Some(task_id));
        assert_eq!(agent.status, AgentStatus::Working);
    }

    #[test]
    fn test_capacity_limits() {
        let mut workshop = WorkshopManager::new();
        
        // Scaffolder has capacity of 1
        let agent1 = Agent::new(AgentRole::Scaffolder, 1);
        let agent2 = Agent::new(AgentRole::Scaffolder, 2);
        
        workshop.register_agent(agent1).unwrap();
        workshop.register_agent(agent2).unwrap();

        let task1 = Task::new("Task 1".to_string(), "Desc".to_string(), AgentRole::Scaffolder, TaskPriority::Normal);
        let _task2 = Task::new("Task 2".to_string(), "Desc".to_string(), AgentRole::Scaffolder, TaskPriority::Normal);

        // First assignment should work
        assert!(workshop.assign_task(task1).is_ok());
        
        // Second assignment should fail due to capacity
        assert!(!workshop.can_assign(AgentRole::Scaffolder));
    }
}