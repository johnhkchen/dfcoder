use crate::*;
use std::time::Duration;
use async_trait::async_trait;

/// Behavior execution engine
#[derive(Debug)]
pub struct BehaviorEngine {
    registry: AgentRegistry,
    event_bus: EventBus,
    execution_context: ExecutionContext,
}

/// Context for behavior execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub current_agent: Option<String>,
    pub current_task: Option<String>,
    pub environment_vars: std::collections::HashMap<String, String>,
    pub working_directory: String,
    pub timeout: Duration,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            current_agent: None,
            current_task: None,
            environment_vars: std::collections::HashMap::new(),
            working_directory: ".".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl BehaviorEngine {
    /// Create a new behavior engine
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();
        registry.register_common_archetypes();
        
        Self {
            registry,
            event_bus: EventBus::new(),
            execution_context: ExecutionContext::default(),
        }
    }
    
    /// Execute a behavior for a specific trigger
    pub async fn execute_behavior(
        &mut self,
        agent_name: &str,
        trigger: TriggerCondition,
    ) -> Result<Option<AgentAction>, DslError> {
        if let Some(agent) = self.registry.get(agent_name) {
            if let Some(action) = agent.handle_trigger(&trigger).await {
                self.execution_context.current_agent = Some(agent_name.to_string());
                
                // Execute the action
                self.execute_action(&action).await?;
                
                // Publish event
                self.event_bus
                    .publish(SystemEvent::AgentStateChanged {
                        agent_id: agent_name.to_string(),
                        old_state: agent.current_state().clone(),
                        new_state: AgentState::Working,
                    })
                    .await
                    .map_err(|e| DslError::EventHandlingFailed(e.to_string()))?;
                
                return Ok(Some(action));
            }
        }
        
        Ok(None)
    }
    
    /// Execute an agent action
    async fn execute_action(&self, action: &AgentAction) -> Result<(), DslError> {
        match action {
            AgentAction::Respond(message) => {
                tracing::info!("Agent response: {}", message);
                Ok(())
            }
            AgentAction::RequestHelp(message) => {
                tracing::warn!("Agent requesting help: {}", message);
                self.event_bus
                    .publish(SystemEvent::SupervisionRequested {
                        agent_id: self.execution_context.current_agent
                            .clone()
                            .unwrap_or_default(),
                        message: message.clone(),
                        context: self.execution_context.current_task
                            .clone()
                            .unwrap_or_default(),
                    })
                    .await
                    .map_err(|e| DslError::AgentActionFailed(e.to_string()))?;
                Ok(())
            }
            AgentAction::ExecuteCommand(command) => {
                tracing::info!("Executing command: {}", command);
                // In a real implementation, this would execute the command
                // For now, we'll just simulate it
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            }
            AgentAction::Monitor(pattern) => {
                tracing::info!("Monitoring files matching: {}", pattern);
                // In a real implementation, this would set up file monitoring
                Ok(())
            }
            AgentAction::AnalyzeCode(description) => {
                tracing::info!("Analyzing code: {}", description);
                // In a real implementation, this would perform code analysis
                Ok(())
            }
            AgentAction::None => Ok(()),
        }
    }
    
    /// Get the agent registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
    
    /// Get a mutable reference to the agent registry
    pub fn registry_mut(&mut self) -> &mut AgentRegistry {
        &mut self.registry
    }
    
    /// Get the event bus
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
    
    /// Set the current working directory
    pub fn set_working_directory(&mut self, path: impl Into<String>) {
        self.execution_context.working_directory = path.into();
    }
    
    /// Set an environment variable
    pub fn set_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.execution_context.environment_vars.insert(key.into(), value.into());
    }
    
    /// Set the execution timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.execution_context.timeout = timeout;
    }
}

impl Default for BehaviorEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Behavior patterns for common agent interactions
pub struct BehaviorPatterns;

impl BehaviorPatterns {
    /// Help-seeking behavior when agent is stuck
    pub fn help_seeking_behavior() -> BehaviorRule {
        BehaviorRule {
            trigger: TriggerCondition::RespondsTo("stuck".to_string()),
            action: AgentAction::RequestHelp(
                "I'm having trouble with this task and need guidance".to_string()
            ),
            description: "Seeks help when stuck on a task".to_string(),
        }
    }
    
    /// Error analysis behavior
    pub fn error_analysis_behavior() -> BehaviorRule {
        BehaviorRule {
            trigger: TriggerCondition::RespondsTo("error".to_string()),
            action: AgentAction::AnalyzeCode("Analyzing error context and potential solutions".to_string()),
            description: "Analyzes errors when they occur".to_string(),
        }
    }
    
    /// Progress reporting behavior
    pub fn progress_reporting_behavior() -> BehaviorRule {
        BehaviorRule {
            trigger: TriggerCondition::DuringSupervision,
            action: AgentAction::Respond("Reporting current progress and next steps".to_string()),
            description: "Reports progress during supervision".to_string(),
        }
    }
    
    /// File monitoring behavior
    pub fn file_monitoring_behavior(pattern: &str) -> BehaviorRule {
        BehaviorRule {
            trigger: TriggerCondition::WhenIdle,
            action: AgentAction::Monitor(pattern.to_string()),
            description: format!("Monitors files matching pattern: {}", pattern),
        }
    }
    
    /// Testing behavior
    pub fn testing_behavior() -> BehaviorRule {
        BehaviorRule {
            trigger: TriggerCondition::RespondsTo("test".to_string()),
            action: AgentAction::ExecuteCommand("cargo test".to_string()),
            description: "Runs tests when testing is mentioned".to_string(),
        }
    }
    
    /// Build behavior
    pub fn build_behavior() -> BehaviorRule {
        BehaviorRule {
            trigger: TriggerCondition::RespondsTo("build".to_string()),
            action: AgentAction::ExecuteCommand("cargo build".to_string()),
            description: "Builds project when build is mentioned".to_string(),
        }
    }
}

/// Behavior scheduler for managing when behaviors should be executed
#[derive(Debug)]
pub struct BehaviorScheduler {
    scheduled_behaviors: Vec<ScheduledBehavior>,
    next_execution_id: u64,
}

#[derive(Debug, Clone)]
struct ScheduledBehavior {
    id: u64,
    agent_name: String,
    trigger: TriggerCondition,
    schedule: Schedule,
    next_execution: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum Schedule {
    /// Execute once at a specific time
    Once(chrono::DateTime<chrono::Utc>),
    /// Execute repeatedly with an interval
    Interval(Duration),
    /// Execute when a condition is met
    WhenCondition(Box<dyn traits::Condition>),
}

impl BehaviorScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_behaviors: Vec::new(),
            next_execution_id: 1,
        }
    }
    
    /// Schedule a behavior for execution
    pub fn schedule_behavior(
        &mut self,
        agent_name: String,
        trigger: TriggerCondition,
        schedule: Schedule,
    ) -> u64 {
        let id = self.next_execution_id;
        self.next_execution_id += 1;
        
        let next_execution = match &schedule {
            Schedule::Once(time) => *time,
            Schedule::Interval(duration) => chrono::Utc::now() + chrono::Duration::from_std(*duration).unwrap(),
            Schedule::WhenCondition(_) => chrono::Utc::now(),
        };
        
        self.scheduled_behaviors.push(ScheduledBehavior {
            id,
            agent_name,
            trigger,
            schedule,
            next_execution,
        });
        
        id
    }
    
    /// Get behaviors that are ready to execute
    pub fn ready_behaviors(&mut self) -> Vec<(String, TriggerCondition)> {
        let now = chrono::Utc::now();
        let mut ready = Vec::new();
        
        for behavior in &mut self.scheduled_behaviors {
            if behavior.next_execution <= now {
                ready.push((behavior.agent_name.clone(), behavior.trigger.clone()));
                
                // Update next execution time for recurring behaviors
                match &behavior.schedule {
                    Schedule::Interval(duration) => {
                        behavior.next_execution = now + chrono::Duration::from_std(*duration).unwrap();
                    }
                    _ => {
                        // Remove one-time schedules after execution
                        // This will be handled by the caller
                    }
                }
            }
        }
        
        // Remove completed one-time behaviors
        self.scheduled_behaviors.retain(|b| {
            matches!(b.schedule, Schedule::Interval(_)) || b.next_execution > now
        });
        
        ready
    }
    
    /// Cancel a scheduled behavior
    pub fn cancel_behavior(&mut self, id: u64) -> bool {
        let original_len = self.scheduled_behaviors.len();
        self.scheduled_behaviors.retain(|b| b.id != id);
        self.scheduled_behaviors.len() != original_len
    }
    
    /// Get the number of scheduled behaviors
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_behaviors.len()
    }
}

impl Default for BehaviorScheduler {
    fn default() -> Self {
        Self::new()
    }
}