use crate::*;
use std::collections::VecDeque;
use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};

/// Event bus for managing system-wide events
#[derive(Debug)]
pub struct EventBus {
    sender: broadcast::Sender<SystemEvent>,
    receiver: broadcast::Receiver<SystemEvent>,
    handlers: std::collections::HashMap<String, Box<dyn EventHandlerWrapper>>,
}

/// Wrapper trait for type-erased event handlers
#[async_trait]
trait EventHandlerWrapper: Send + Sync {
    async fn handle_event(&self, event: &SystemEvent) -> Result<(), EventError>;
}

/// Implementation of EventHandlerWrapper for typed handlers
#[async_trait]
impl<T, H> EventHandlerWrapper for H
where
    T: Event + Clone + Send + Sync + 'static,
    H: traits::EventHandler<T> + Send + Sync + 'static,
{
    async fn handle_event(&self, event: &SystemEvent) -> Result<(), EventError> {
        // Type-safe event handling would require more sophisticated dispatch
        // For now, we'll use a simplified approach
        Ok(())
    }
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel(1000);
        Self {
            sender,
            receiver,
            handlers: std::collections::HashMap::new(),
        }
    }
    
    /// Publish an event
    pub async fn publish(&self, event: SystemEvent) -> Result<(), EventError> {
        self.sender
            .send(event)
            .map_err(|e| EventError::ProcessingFailed(format!("Failed to send event: {}", e)))?;
        Ok(())
    }
    
    /// Subscribe to events with a callback
    pub async fn subscribe<F>(&self, mut callback: F) -> Result<(), EventError>
    where
        F: FnMut(SystemEvent) -> Result<(), EventError> + Send + 'static,
    {
        let mut receiver = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let Err(e) = callback(event) {
                    tracing::error!("Event handler error: {}", e);
                }
            }
        });
        Ok(())
    }
    
    /// Register a typed event handler
    pub fn register_handler<T, H>(&mut self, name: String, handler: H)
    where
        T: Event + Clone + Send + Sync + 'static,
        H: traits::EventHandler<T> + Send + Sync + 'static,
    {
        self.handlers.insert(name, Box::new(handler));
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event flow definitions for common patterns
pub struct EventFlows;

impl EventFlows {
    /// Agent supervision flow
    pub fn supervision_flow() -> Vec<SystemEvent> {
        vec![
            SystemEvent::AgentStateChanged {
                agent_id: "example_agent".to_string(),
                old_state: AgentState::Working,
                new_state: AgentState::Stuck,
            },
            SystemEvent::SupervisionRequested {
                agent_id: "example_agent".to_string(),
                message: "I need help with this task".to_string(),
                context: "Working on complex algorithm".to_string(),
            },
        ]
    }
    
    /// Task completion flow
    pub fn task_completion_flow() -> Vec<SystemEvent> {
        vec![
            SystemEvent::AgentStateChanged {
                agent_id: "example_agent".to_string(),
                old_state: AgentState::Working,
                new_state: AgentState::Idle,
            },
            SystemEvent::TaskCompleted {
                agent_id: "example_agent".to_string(),
                task_id: "task_123".to_string(),
                result: TaskResult::Success,
            },
        ]
    }
    
    /// Error handling flow
    pub fn error_handling_flow() -> Vec<SystemEvent> {
        vec![
            SystemEvent::ErrorOccurred {
                agent_id: "example_agent".to_string(),
                error_message: "Compilation failed".to_string(),
                context: "Building Rust project".to_string(),
            },
            SystemEvent::SupervisionRequested {
                agent_id: "example_agent".to_string(),
                message: "Need help resolving compilation errors".to_string(),
                context: "Multiple type errors in main.rs".to_string(),
            },
        ]
    }
}

/// Event patterns for matching and filtering
#[derive(Debug, Clone)]
pub enum EventPattern {
    /// Match events from specific agent
    FromAgent(String),
    /// Match events of specific type
    OfType(String),
    /// Match events with specific content
    Contains(String),
    /// Composite pattern with AND logic
    And(Vec<EventPattern>),
    /// Composite pattern with OR logic
    Or(Vec<EventPattern>),
}

impl EventPattern {
    /// Check if an event matches this pattern
    pub fn matches(&self, event: &SystemEvent) -> bool {
        match self {
            EventPattern::FromAgent(agent_id) => {
                match event {
                    SystemEvent::AgentStateChanged { agent_id: id, .. } => id == agent_id,
                    SystemEvent::SupervisionRequested { agent_id: id, .. } => id == agent_id,
                    SystemEvent::TaskCompleted { agent_id: id, .. } => id == agent_id,
                    SystemEvent::ErrorOccurred { agent_id: id, .. } => id == agent_id,
                }
            }
            EventPattern::OfType(event_type) => {
                let actual_type = match event {
                    SystemEvent::AgentStateChanged { .. } => "AgentStateChanged",
                    SystemEvent::SupervisionRequested { .. } => "SupervisionRequested",
                    SystemEvent::TaskCompleted { .. } => "TaskCompleted",
                    SystemEvent::ErrorOccurred { .. } => "ErrorOccurred",
                };
                actual_type == event_type
            }
            EventPattern::Contains(content) => {
                let event_text = format!("{:?}", event);
                event_text.contains(content)
            }
            EventPattern::And(patterns) => {
                patterns.iter().all(|p| p.matches(event))
            }
            EventPattern::Or(patterns) => {
                patterns.iter().any(|p| p.matches(event))
            }
        }
    }
}

/// Event queue for buffering and processing events
#[derive(Debug)]
pub struct EventQueue {
    events: VecDeque<SystemEvent>,
    max_size: usize,
}

impl EventQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_size,
        }
    }
    
    /// Add an event to the queue
    pub fn push(&mut self, event: SystemEvent) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
    
    /// Get the next event from the queue
    pub fn pop(&mut self) -> Option<SystemEvent> {
        self.events.pop_front()
    }
    
    /// Peek at the next event without removing it
    pub fn peek(&self) -> Option<&SystemEvent> {
        self.events.front()
    }
    
    /// Filter events by pattern
    pub fn filter(&self, pattern: &EventPattern) -> Vec<&SystemEvent> {
        self.events
            .iter()
            .filter(|event| pattern.matches(event))
            .collect()
    }
    
    /// Get the number of events in the queue
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    /// Clear all events from the queue
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Event aggregator for collecting and analyzing events
#[derive(Debug, Default)]
pub struct EventAggregator {
    events_by_agent: std::collections::HashMap<String, Vec<SystemEvent>>,
    events_by_type: std::collections::HashMap<String, Vec<SystemEvent>>,
    recent_events: VecDeque<SystemEvent>,
}

impl EventAggregator {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an event to the aggregator
    pub fn add_event(&mut self, event: SystemEvent) {
        // Group by agent
        if let Some(agent_id) = self.extract_agent_id(&event) {
            self.events_by_agent
                .entry(agent_id)
                .or_default()
                .push(event.clone());
        }
        
        // Group by type
        let event_type = self.event_type_name(&event);
        self.events_by_type
            .entry(event_type)
            .or_default()
            .push(event.clone());
        
        // Keep recent events (last 100)
        if self.recent_events.len() >= 100 {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(event);
    }
    
    /// Get events for a specific agent
    pub fn events_for_agent(&self, agent_id: &str) -> Option<&Vec<SystemEvent>> {
        self.events_by_agent.get(agent_id)
    }
    
    /// Get events of a specific type
    pub fn events_of_type(&self, event_type: &str) -> Option<&Vec<SystemEvent>> {
        self.events_by_type.get(event_type)
    }
    
    /// Get recent events
    pub fn recent_events(&self) -> &VecDeque<SystemEvent> {
        &self.recent_events
    }
    
    /// Get agent activity summary
    pub fn agent_activity_summary(&self) -> std::collections::HashMap<String, usize> {
        self.events_by_agent
            .iter()
            .map(|(agent_id, events)| (agent_id.clone(), events.len()))
            .collect()
    }
    
    fn extract_agent_id(&self, event: &SystemEvent) -> Option<String> {
        match event {
            SystemEvent::AgentStateChanged { agent_id, .. } => Some(agent_id.clone()),
            SystemEvent::SupervisionRequested { agent_id, .. } => Some(agent_id.clone()),
            SystemEvent::TaskCompleted { agent_id, .. } => Some(agent_id.clone()),
            SystemEvent::ErrorOccurred { agent_id, .. } => Some(agent_id.clone()),
        }
    }
    
    fn event_type_name(&self, event: &SystemEvent) -> String {
        match event {
            SystemEvent::AgentStateChanged { .. } => "AgentStateChanged".to_string(),
            SystemEvent::SupervisionRequested { .. } => "SupervisionRequested".to_string(),
            SystemEvent::TaskCompleted { .. } => "TaskCompleted".to_string(),
            SystemEvent::ErrorOccurred { .. } => "ErrorOccurred".to_string(),
        }
    }
}