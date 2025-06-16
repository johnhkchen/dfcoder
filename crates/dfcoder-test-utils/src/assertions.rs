use std::time::Duration;
use dfcoder_types::*;
use crate::{MockAgent, MockSupervisor};

/// Assertion helpers for test scenarios
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that an agent is in a specific state
    pub fn agent_is_in_state(agent: &MockAgent, expected_status: AgentStatus) {
        assert_eq!(
            agent.status, expected_status,
            "Agent '{}' expected to be {:?}, but was {:?}",
            agent.name, expected_status, agent.status
        );
    }
    
    /// Assert that an agent has requested help
    pub fn agent_has_requested_help(agent: &MockAgent) {
        assert!(
            agent.needs_help(),
            "Agent '{}' should have requested help but didn't",
            agent.name
        );
    }
    
    /// Assert that an agent has not requested help
    pub fn agent_has_not_requested_help(agent: &MockAgent) {
        assert!(
            !agent.needs_help(),
            "Agent '{}' should not have requested help but did",
            agent.name
        );
    }
    
    /// Assert that an agent has been working for a specific duration
    pub fn agent_has_been_working_for(agent: &MockAgent, expected_duration: Duration) {
        let actual_duration = agent.last_activity.elapsed();
        assert!(
            actual_duration >= expected_duration,
            "Agent '{}' should have been working for at least {:?}, but only worked for {:?}",
            agent.name, expected_duration, actual_duration
        );
    }
    
    /// Assert that supervisor has active dialogues
    pub fn supervisor_has_active_dialogues(supervisor: &MockSupervisor) {
        assert!(
            supervisor.has_active_dialogues(),
            "Supervisor should have active dialogues but doesn't"
        );
    }
    
    /// Assert that supervisor has no active dialogues
    pub fn supervisor_has_no_active_dialogues(supervisor: &MockSupervisor) {
        assert!(
            !supervisor.has_active_dialogues(),
            "Supervisor should not have active dialogues but does"
        );
    }
    
    /// Assert that supervisor has dialogue with specific agent
    pub fn supervisor_has_dialogue_with(supervisor: &MockSupervisor, agent_id: &str) {
        assert!(
            supervisor.get_dialogue(agent_id).is_some(),
            "Supervisor should have dialogue with agent '{}' but doesn't",
            agent_id
        );
    }
    
    /// Assert that a dialogue has specific number of options
    pub fn dialogue_has_options(supervisor: &MockSupervisor, agent_id: &str, expected_count: usize) {
        if let Some(dialogue) = supervisor.get_dialogue(agent_id) {
            assert_eq!(
                dialogue.options.len(), expected_count,
                "Dialogue with agent '{}' should have {} options but has {}",
                agent_id, expected_count, dialogue.options.len()
            );
        } else {
            panic!("No dialogue found with agent '{}'", agent_id);
        }
    }
    
    /// Assert that dialogue contains specific option text
    pub fn dialogue_contains_option(supervisor: &MockSupervisor, agent_id: &str, option_text: &str) {
        if let Some(dialogue) = supervisor.get_dialogue(agent_id) {
            let found = dialogue.options.iter().any(|opt| opt.text.contains(option_text));
            assert!(
                found,
                "Dialogue with agent '{}' should contain option with text '{}' but doesn't",
                agent_id, option_text
            );
        } else {
            panic!("No dialogue found with agent '{}'", agent_id);
        }
    }
    
    /// Assert that agent metrics meet expectations
    pub fn agent_metrics_meet_expectations(agent: &MockAgent, min_tasks: u32, max_errors: u32) {
        assert!(
            agent.metrics.tasks_completed >= min_tasks,
            "Agent '{}' should have completed at least {} tasks but only completed {}",
            agent.name, min_tasks, agent.metrics.tasks_completed
        );
        
        assert!(
            agent.metrics.errors_encountered <= max_errors,
            "Agent '{}' should have encountered at most {} errors but encountered {}",
            agent.name, max_errors, agent.metrics.errors_encountered
        );
    }
    
    /// Assert that response time is within acceptable bounds
    pub fn response_time_is_acceptable(agent: &MockAgent, max_response_time_ms: u64) {
        assert!(
            agent.metrics.response_time_ms <= max_response_time_ms,
            "Agent '{}' response time {}ms exceeds maximum {}ms",
            agent.name, agent.metrics.response_time_ms, max_response_time_ms
        );
    }
}

/// Fluent assertion builder for more readable tests
pub struct AssertionBuilder<'a> {
    agent: &'a MockAgent,
}

impl<'a> AssertionBuilder<'a> {
    pub fn new(agent: &'a MockAgent) -> Self {
        Self { agent }
    }
    
    pub fn is_in_state(self, status: AgentStatus) -> Self {
        TestAssertions::agent_is_in_state(self.agent, status);
        self
    }
    
    pub fn has_requested_help(self) -> Self {
        TestAssertions::agent_has_requested_help(self.agent);
        self
    }
    
    pub fn has_not_requested_help(self) -> Self {
        TestAssertions::agent_has_not_requested_help(self.agent);
        self
    }
    
    pub fn has_been_working_for(self, duration: Duration) -> Self {
        TestAssertions::agent_has_been_working_for(self.agent, duration);
        self
    }
    
    pub fn has_completed_at_least(self, tasks: u32) -> Self {
        assert!(
            self.agent.metrics.tasks_completed >= tasks,
            "Agent '{}' should have completed at least {} tasks",
            self.agent.name, tasks
        );
        self
    }
    
    pub fn has_encountered_at_most(self, errors: u32) -> Self {
        assert!(
            self.agent.metrics.errors_encountered <= errors,
            "Agent '{}' should have encountered at most {} errors",
            self.agent.name, errors
        );
        self
    }
}

/// Trait to add assertion methods to MockAgent
pub trait AgentAssertions {
    fn assert(&self) -> AssertionBuilder;
}

impl AgentAssertions for MockAgent {
    fn assert(&self) -> AssertionBuilder {
        AssertionBuilder::new(self)
    }
}

/// Macros for even more readable assertions
#[macro_export]
macro_rules! assert_agent {
    ($agent:expr, is $status:expr) => {
        TestAssertions::agent_is_in_state($agent, $status)
    };
    ($agent:expr, has requested help) => {
        TestAssertions::agent_has_requested_help($agent)
    };
    ($agent:expr, has not requested help) => {
        TestAssertions::agent_has_not_requested_help($agent)
    };
    ($agent:expr, has been working for $duration:expr) => {
        TestAssertions::agent_has_been_working_for($agent, $duration)
    };
}

#[macro_export]
macro_rules! assert_supervisor {
    ($supervisor:expr, has active dialogues) => {
        TestAssertions::supervisor_has_active_dialogues($supervisor)
    };
    ($supervisor:expr, has no active dialogues) => {
        TestAssertions::supervisor_has_no_active_dialogues($supervisor)
    };
    ($supervisor:expr, has dialogue with $agent_id:expr) => {
        TestAssertions::supervisor_has_dialogue_with($supervisor, $agent_id)
    };
}