//! Test utilities and framework for DFCoder scenarios
//! 
//! This crate provides a natural language DSL for writing test scenarios
//! that can be used to validate agent behaviors and system interactions.

use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use futures::Future;
use tokio::sync::{Mutex, RwLock};
use dfcoder_core::{Agent, AgentRole};

pub use scenario::*;
pub use agent_mock::*;
pub use supervisor_mock::*;
pub use assertions::*;
pub use pane_mock::*;

mod scenario;
mod agent_mock;
mod supervisor_mock;
mod assertions;
mod pane_mock;

/// Test scenario builder for natural language test descriptions
pub struct TestScenario {
    name: String,
    state: Arc<RwLock<ScenarioState>>,
    timeline: Arc<Mutex<Vec<ScenarioEvent>>>,
}

#[derive(Debug, Clone)]
pub struct ScenarioState {
    pub agents: Vec<MockAgent>,
    pub supervisor: MockSupervisor,
    pub panes: Vec<MockPane>,
    pub start_time: Instant,
    pub current_time: Instant,
}

#[derive(Debug, Clone)]
pub struct ScenarioEvent {
    pub timestamp: Instant,
    pub event_type: EventType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum EventType {
    AgentAction,
    SupervisorAction,
    PaneUpdate,
    UserInput,
    SystemEvent,
}

impl TestScenario {
    /// Create a new test scenario with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            name: name.into(),
            state: Arc::new(RwLock::new(ScenarioState {
                agents: Vec::new(),
                supervisor: MockSupervisor::new(),
                panes: Vec::new(),
                start_time: now,
                current_time: now,
            })),
            timeline: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Set up initial conditions for the scenario
    pub async fn given<F, Fut>(&self, setup: F) -> &Self
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        setup().await;
        self.record_event(EventType::SystemEvent, "Given conditions established").await;
        self
    }
    
    /// Trigger the event being tested
    pub async fn when<F, Fut>(&self, trigger: F) -> &Self
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        trigger().await;
        self.record_event(EventType::SystemEvent, "When condition triggered").await;
        self
    }
    
    /// Verify the expected outcome
    pub async fn then<F, Fut>(&self, verification: F) -> &Self
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        verification().await;
        self.record_event(EventType::SystemEvent, "Then verification completed").await;
        self
    }
    
    /// Assert that the scenario completed successfully
    pub async fn assert_success(&self) {
        let timeline = self.timeline.lock().await;
        let _state = self.state.read().await;
        
        // Verify timeline has expected structure
        let given_events = timeline.iter().filter(|e| matches!(e.event_type, EventType::SystemEvent)).count();
        assert!(given_events >= 3, "Scenario should have given/when/then events");
        
        tracing::info!("Scenario '{}' completed successfully with {} events", 
                      self.name, timeline.len());
    }
    
    /// Add an agent to the scenario
    pub async fn add_agent(&self, agent: MockAgent) {
        let mut state = self.state.write().await;
        state.agents.push(agent);
        self.record_event(EventType::AgentAction, "Agent added to scenario").await;
    }
    
    /// Add a pane to the scenario
    pub async fn add_pane(&self, pane: MockPane) {
        let mut state = self.state.write().await;
        state.panes.push(pane);
        self.record_event(EventType::PaneUpdate, "Pane added to scenario").await;
    }
    
    /// Simulate time passing
    pub async fn advance_time(&self, duration: Duration) {
        let mut state = self.state.write().await;
        state.current_time += duration;
        self.record_event(EventType::SystemEvent, 
                         &format!("Time advanced by {:?}", duration)).await;
    }
    
    /// Wait for a condition to be met or timeout
    pub async fn wait_for<F, Fut>(&self, condition: F, timeout: Duration) -> bool
    where
        F: Fn() -> Fut,
        Fut: Future<Output = bool>,
    {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if condition().await {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        false
    }
    
    async fn record_event(&self, event_type: EventType, description: &str) {
        let mut timeline = self.timeline.lock().await;
        timeline.push(ScenarioEvent {
            timestamp: Instant::now(),
            event_type,
            description: description.to_string(),
        });
    }
}

/// Convenient builder functions for common test patterns
impl TestScenario {
    /// Create a scenario with an agent working on a task
    pub async fn with_working_agent(name: impl Into<String>, task_duration: Duration) -> Self {
        let scenario = Self::new(name);
        let agent = MockAgent::new("test_agent")
            .with_current_task("complex task")
            .working_for(task_duration);
        scenario.add_agent(agent).await;
        scenario
    }
    
    /// Create a scenario with a stuck agent
    pub async fn with_stuck_agent(name: impl Into<String>) -> Self {
        let scenario = Self::new(name);
        let agent = MockAgent::new("stuck_agent")
            .with_status(dfcoder_core::AgentStatus::Stuck)
            .with_current_task("impossible task");
        scenario.add_agent(agent).await;
        scenario
    }
    
    /// Create a scenario with supervisor intervention needed
    pub async fn with_supervision_needed(name: impl Into<String>) -> Self {
        let scenario = Self::new(name);
        let agent = MockAgent::new("help_needed_agent")
            .requesting_help("Need guidance on approach");
        scenario.add_agent(agent).await;
        scenario
    }
}