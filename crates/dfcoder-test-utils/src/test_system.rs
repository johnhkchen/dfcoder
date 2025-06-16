//! Test system for scenario-based testing of DFCoder components
//! 
//! Provides a high-level API for testing agent behaviors and system interactions.

use dfcoder_core::*;
use dfcoder_baml::*;
use std::collections::HashMap;
use std::time::Duration;

/// High-level test system for simulating DFCoder scenarios
pub struct TestSystem {
    workshop: WorkshopManager,
    supervision: SupervisionSystem,
    agents: HashMap<AgentId, Agent>,
    simulated_time: Duration,
    agent_outputs: HashMap<AgentId, Vec<String>>,
}

/// Represents a spawned agent in the test system
pub struct SpawnedAgent {
    pub id: AgentId,
    pub role: AgentRole,
}

impl TestSystem {
    /// Create a new test system
    pub fn new() -> Self {
        Self {
            workshop: WorkshopManager::new(),
            supervision: SupervisionSystem::new(),
            agents: HashMap::new(),
            simulated_time: Duration::from_secs(0),
            agent_outputs: HashMap::new(),
        }
    }

    /// Spawn a new agent with the given role
    pub fn spawn_agent(&mut self, role: AgentRole) -> SpawnedAgent {
        let pane_id = self.agents.len() as u32 + 1;
        let agent = Agent::new(role.clone(), pane_id);
        let agent_id = agent.id.clone();
        
        // Register with workshop
        self.workshop.register_agent(agent.clone()).unwrap();
        
        // Store in our tracking
        self.agents.insert(agent_id.clone(), agent);
        self.agent_outputs.insert(agent_id.clone(), Vec::new());
        
        SpawnedAgent {
            id: agent_id,
            role,
        }
    }

    /// Assign a task to an agent and return (task_id, actual_assigned_agent_id)
    pub fn assign_task(&mut self, _requested_agent_id: AgentId, task_description: &str) -> TaskId {
        // For simplicity in testing, we'll create a task for any available agent
        // and let the workshop manager assign it to whoever is available
        let task = Task::new(
            task_description.to_string(),
            format!("Test task: {}", task_description),
            AgentRole::Implementer, // Default role for testing
            TaskPriority::Normal,
        );
        let task_id = task.id.clone();
        
        // Queue and assign task
        self.workshop.queue_task(task);
        let assignment = self.workshop.try_assign_next_task().unwrap();
        
        if assignment.is_none() {
            panic!("Failed to assign task - no available agents");
        }
        
        task_id
    }

    /// Assign a task to a specific role
    pub fn assign_task_to_role(&mut self, role: AgentRole, task_description: &str) -> (TaskId, AgentId) {
        let task = Task::new(
            task_description.to_string(),
            format!("Test task: {}", task_description),
            role,
            TaskPriority::Normal,
        );
        let task_id = task.id.clone();
        
        // Directly assign the task
        let assigned_agent = self.workshop.assign_task(task).expect("Failed to assign task");
        
        (task_id, assigned_agent)
    }

    /// Simulate agent output (as if the agent wrote something)
    pub fn simulate_output(&mut self, agent_id: AgentId, output: &str) {
        // Store the output
        self.agent_outputs.entry(agent_id.clone()).or_default().push(output.to_string());
        
        // Trigger supervision check if needed
        if let Some(agent) = self.agents.get(&agent_id) {
            // This would trigger supervision in a real scenario
            let _ = futures::executor::block_on(
                self.supervision.check_supervision_need(agent, output)
            );
        }
    }

    /// Advance simulated time
    pub fn advance_time(&mut self, duration: Duration) {
        self.simulated_time += duration;
        
        // Clean up any expired supervision requests
        self.supervision.cleanup_expired_requests();
    }

    /// Check if there's an active supervision request for any agent
    pub fn has_supervision_request(&self) -> bool {
        !self.supervision.get_all_active_requests().is_empty()
    }

    /// Get the first supervision request (for testing)
    pub fn get_supervision_request(&self) -> Option<&SupervisionRequest> {
        self.supervision.get_all_active_requests().first().copied()
    }

    /// Get supervision request for a specific agent
    pub fn get_supervision_request_for(&self, agent_id: &AgentId) -> Option<&SupervisionRequest> {
        self.supervision.get_active_request(agent_id)
    }

    /// Get current workshop status
    pub fn get_workshop_status(&mut self) -> WorkshopStatus {
        self.workshop.get_status()
    }

    /// Get agent by ID (returns the workshop's version which has current state)
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<&Agent> {
        self.workshop.get_agent(agent_id)
    }

    /// Complete a task for an agent
    pub fn complete_task(&mut self, agent_id: AgentId, task_id: TaskId) -> Result<(), WorkshopError> {
        self.workshop.complete_task(agent_id, task_id)
    }

    /// Fail a task for an agent
    pub fn fail_task(&mut self, agent_id: AgentId, task_id: TaskId, error: String) -> Result<(), WorkshopError> {
        self.workshop.fail_task(agent_id, task_id, error)
    }

    /// Get all outputs for an agent
    pub fn get_agent_outputs(&self, agent_id: &AgentId) -> Vec<String> {
        self.agent_outputs.get(agent_id).cloned().unwrap_or_default()
    }

    /// Check if an agent is stuck (has supervision request)
    pub fn is_agent_stuck(&self, agent_id: &AgentId) -> bool {
        self.supervision.get_active_request(agent_id).is_some()
    }

    /// Respond to a supervision request
    pub async fn respond_to_supervision(
        &mut self, 
        agent_id: &AgentId, 
        option_id: u32
    ) -> Result<SupervisionAction, SupervisionError> {
        self.supervision.handle_supervision_response(agent_id, option_id).await
    }

    /// Get current simulated time
    pub fn elapsed_time(&self) -> Duration {
        self.simulated_time
    }

    /// Check if workshop is at capacity for a role
    pub fn is_at_capacity(&self, role: AgentRole) -> bool {
        !self.workshop.can_assign(role)
    }

    /// Get number of active agents
    pub fn active_agent_count(&mut self) -> usize {
        self.workshop.get_status().active_agents
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.workshop.get_queue().len()
    }
}

impl Default for TestSystem {
    fn default() -> Self {
        Self::new()
    }
}