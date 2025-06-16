use crate::*;
use std::collections::HashMap;
use async_trait::async_trait;

/// Agent definition using natural language DSL
#[derive(Debug)]
pub struct AgentDefinition {
    pub name: String,
    pub behaviors: Vec<BehaviorRule>,
    pub state: AgentState,
}

/// A behavior rule for an agent
#[derive(Debug, Clone)]
pub struct BehaviorRule {
    pub trigger: TriggerCondition,
    pub action: AgentAction,
    pub description: String,
}

/// Pre-defined agent archetypes with common behaviors
pub struct AgentArchetypes;

impl AgentArchetypes {
    /// Rust expert agent with type system knowledge
    pub fn rust_expert() -> AgentDefinition {
        AgentDefinition {
            name: "RustExpert".to_string(),
            state: AgentState::default(),
            behaviors: vec![
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("type error".to_string()),
                    action: AgentAction::AnalyzeCode("Analyzing type error...".to_string()),
                    description: "Responds to type errors with careful analysis".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::WhenIdle,
                    action: AgentAction::Monitor("*.rs".to_string()),
                    description: "Monitors Rust files when idle".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::DuringSupervision,
                    action: AgentAction::Respond("Providing context within 10 lines".to_string()),
                    description: "Provides context during supervision".to_string(),
                },
            ],
        }
    }
    
    /// JavaScript/TypeScript expert agent
    pub fn typescript_expert() -> AgentDefinition {
        AgentDefinition {
            name: "TypeScriptExpert".to_string(),
            state: AgentState::default(),
            behaviors: vec![
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("typescript".to_string()),
                    action: AgentAction::AnalyzeCode("Analyzing TypeScript code...".to_string()),
                    description: "Responds to TypeScript-related queries".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("npm error".to_string()),
                    action: AgentAction::ExecuteCommand("npm install".to_string()),
                    description: "Handles npm dependency issues".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::WhenIdle,
                    action: AgentAction::Monitor("*.ts,*.tsx,*.js,*.jsx".to_string()),
                    description: "Monitors JS/TS files when idle".to_string(),
                },
            ],
        }
    }
    
    /// General purpose coding assistant
    pub fn coding_assistant() -> AgentDefinition {
        AgentDefinition {
            name: "CodingAssistant".to_string(),
            state: AgentState::default(),
            behaviors: vec![
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("help".to_string()),
                    action: AgentAction::RequestHelp("How can I assist you?".to_string()),
                    description: "Responds to general help requests".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("stuck".to_string()),
                    action: AgentAction::RequestHelp("I need guidance on the current task".to_string()),
                    description: "Requests help when stuck".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::DuringSupervision,
                    action: AgentAction::Respond("Explaining current approach...".to_string()),
                    description: "Explains approach during supervision".to_string(),
                },
            ],
        }
    }
    
    /// Testing specialist agent
    pub fn test_specialist() -> AgentDefinition {
        AgentDefinition {
            name: "TestSpecialist".to_string(),
            state: AgentState::default(),
            behaviors: vec![
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("test".to_string()),
                    action: AgentAction::ExecuteCommand("cargo test".to_string()),
                    description: "Runs tests when test-related queries occur".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::RespondsTo("coverage".to_string()),
                    action: AgentAction::ExecuteCommand("cargo tarpaulin".to_string()),
                    description: "Checks test coverage".to_string(),
                },
                BehaviorRule {
                    trigger: TriggerCondition::WhenIdle,
                    action: AgentAction::Monitor("*test*.rs,*spec*.js".to_string()),
                    description: "Monitors test files when idle".to_string(),
                },
            ],
        }
    }
}

/// Agent behavior implementation
#[async_trait]
impl traits::AgentBehavior for AgentDefinition {
    async fn handle_trigger(&self, trigger: &TriggerCondition) -> Option<AgentAction> {
        for behavior in &self.behaviors {
            if self.trigger_matches(&behavior.trigger, trigger) {
                tracing::info!("Agent '{}' triggered: {}", self.name, behavior.description);
                return Some(behavior.action.clone());
            }
        }
        None
    }
    
    fn current_state(&self) -> &AgentState {
        &self.state
    }
    
    fn update_state(&mut self, new_state: AgentState) {
        self.state = new_state;
    }
}

impl AgentDefinition {
    /// Check if a trigger condition matches
    fn trigger_matches(&self, rule_trigger: &TriggerCondition, actual_trigger: &TriggerCondition) -> bool {
        match (rule_trigger, actual_trigger) {
            (TriggerCondition::RespondsTo(pattern), TriggerCondition::RespondsTo(input)) => {
                input.to_lowercase().contains(&pattern.to_lowercase())
            }
            (TriggerCondition::WhenIdle, TriggerCondition::WhenIdle) => true,
            (TriggerCondition::DuringSupervision, TriggerCondition::DuringSupervision) => true,
            _ => false,
        }
    }
    
    /// Add a new behavior rule
    pub fn add_behavior(&mut self, trigger: TriggerCondition, action: AgentAction, description: String) {
        self.behaviors.push(BehaviorRule {
            trigger,
            action,
            description,
        });
    }
    
    /// Remove a behavior by description
    pub fn remove_behavior(&mut self, description: &str) -> bool {
        let original_len = self.behaviors.len();
        self.behaviors.retain(|b| b.description != description);
        self.behaviors.len() != original_len
    }
    
    /// Get all behaviors matching a trigger type
    pub fn behaviors_for_trigger(&self, trigger: &TriggerCondition) -> Vec<&BehaviorRule> {
        self.behaviors
            .iter()
            .filter(|b| self.trigger_matches(&b.trigger, trigger))
            .collect()
    }
}

/// Agent registry for managing multiple agents
#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new agent
    pub fn register(&mut self, agent: AgentDefinition) {
        let name = agent.name.clone();
        self.agents.insert(name, agent);
    }
    
    /// Get an agent by name
    pub fn get(&self, name: &str) -> Option<&AgentDefinition> {
        self.agents.get(name)
    }
    
    /// Get a mutable reference to an agent
    pub fn get_mut(&mut self, name: &str) -> Option<&mut AgentDefinition> {
        self.agents.get_mut(name)
    }
    
    /// List all registered agent names
    pub fn agent_names(&self) -> Vec<&String> {
        self.agents.keys().collect()
    }
    
    /// Find agents that can handle a specific trigger
    pub fn agents_for_trigger(&self, trigger: &TriggerCondition) -> Vec<&AgentDefinition> {
        self.agents
            .values()
            .filter(|agent| !agent.behaviors_for_trigger(trigger).is_empty())
            .collect()
    }
    
    /// Register all common agent archetypes
    pub fn register_common_archetypes(&mut self) {
        self.register(AgentArchetypes::rust_expert());
        self.register(AgentArchetypes::typescript_expert());
        self.register(AgentArchetypes::coding_assistant());
        self.register(AgentArchetypes::test_specialist());
    }
}

/// Builder for creating custom agents
pub struct AgentBuilder {
    name: String,
    behaviors: Vec<BehaviorRule>,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            behaviors: Vec::new(),
        }
    }
    
    pub fn responds_to(mut self, pattern: impl Into<String>, action: AgentAction) -> Self {
        self.behaviors.push(BehaviorRule {
            trigger: TriggerCondition::RespondsTo(pattern.into()),
            action,
            description: format!("Responds to '{}'", pattern.into()),
        });
        self
    }
    
    pub fn when_idle(mut self, action: AgentAction) -> Self {
        self.behaviors.push(BehaviorRule {
            trigger: TriggerCondition::WhenIdle,
            action,
            description: "When idle behavior".to_string(),
        });
        self
    }
    
    pub fn during_supervision(mut self, action: AgentAction) -> Self {
        self.behaviors.push(BehaviorRule {
            trigger: TriggerCondition::DuringSupervision,
            action,
            description: "During supervision behavior".to_string(),
        });
        self
    }
    
    pub fn build(self) -> AgentDefinition {
        AgentDefinition {
            name: self.name,
            behaviors: self.behaviors,
            state: AgentState::default(),
        }
    }
}