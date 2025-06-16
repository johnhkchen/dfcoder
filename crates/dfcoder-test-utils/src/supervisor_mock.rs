use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::agent_mock::HelpRequest;

/// Mock supervisor for testing dialogue scenarios
#[derive(Debug, Clone)]
pub struct MockSupervisor {
    pub active_dialogues: HashMap<String, DialogueSession>,
    pub response_queue: VecDeque<SupervisorResponse>,
    pub auto_respond: bool,
    pub response_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DialogueSession {
    pub agent_id: String,
    pub context: DialogueContext,
    pub options: Vec<DialogueOption>,
    #[serde(skip)]
    pub created_at: Instant,
    #[serde(skip)]
    pub last_interaction: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueContext {
    pub situation: String,
    pub relevant_code: String,
    pub error_message: Option<String>,
    pub task_description: String,
    #[serde(with = "dfcoder_types::duration_serde")]
    pub time_stuck: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    pub id: u32,
    pub text: String,
    pub action: DialogueAction,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueAction {
    ProvideGuidance(String),
    RequestMoreInfo,
    TakeOver,
    IgnoreForNow,
    EscalateToHuman,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SupervisorResponse {
    pub agent_id: String,
    pub chosen_option: u32,
    pub custom_message: Option<String>,
    #[serde(skip)]
    pub timestamp: Instant,
}

impl MockSupervisor {
    /// Create a new mock supervisor
    pub fn new() -> Self {
        Self {
            active_dialogues: HashMap::new(),
            response_queue: VecDeque::new(),
            auto_respond: false,
            response_delay_ms: 1000,
        }
    }
    
    /// Enable automatic responses for testing
    pub fn with_auto_respond(mut self, delay_ms: u64) -> Self {
        self.auto_respond = true;
        self.response_delay_ms = delay_ms;
        self
    }
    
    /// Queue a specific response for an agent
    pub fn queue_response(mut self, agent_id: impl Into<String>, option_id: u32) -> Self {
        self.response_queue.push_back(SupervisorResponse {
            agent_id: agent_id.into(),
            chosen_option: option_id,
            custom_message: None,
            timestamp: Instant::now(),
        });
        self
    }
    
    /// Handle a help request from an agent
    pub async fn handle_help_request(&mut self, agent_id: String, request: HelpRequest) {
        let context = DialogueContext {
            situation: request.message.clone(),
            relevant_code: "// Mock code context".to_string(),
            error_message: None,
            task_description: request.context.clone(),
            time_stuck: request.timestamp.elapsed(),
        };
        
        let options = self.generate_dialogue_options(&context);
        
        let session = DialogueSession {
            agent_id: agent_id.clone(),
            context,
            options,
            created_at: Instant::now(),
            last_interaction: Instant::now(),
        };
        
        self.active_dialogues.insert(agent_id, session);
    }
    
    /// Generate contextual dialogue options
    fn generate_dialogue_options(&self, context: &DialogueContext) -> Vec<DialogueOption> {
        let mut options = Vec::new();
        
        // Analyze the situation and provide relevant options
        match context.situation.to_lowercase() {
            s if s.contains("stuck") => {
                options.push(DialogueOption {
                    id: 1,
                    text: "Provide debugging guidance".to_string(),
                    action: DialogueAction::ProvideGuidance(
                        "Let's break this down step by step. Can you show me the exact error?".to_string()
                    ),
                    icon: "🔍".to_string(),
                });
                
                options.push(DialogueOption {
                    id: 2,
                    text: "Request more context".to_string(),
                    action: DialogueAction::RequestMoreInfo,
                    icon: "❓".to_string(),
                });
            }
            s if s.contains("error") => {
                options.push(DialogueOption {
                    id: 1,
                    text: "Help analyze the error".to_string(),
                    action: DialogueAction::ProvideGuidance(
                        "Let me help you understand this error. Here's what it means...".to_string()
                    ),
                    icon: "🛠️".to_string(),
                });
            }
            _ => {
                options.push(DialogueOption {
                    id: 1,
                    text: "Provide general guidance".to_string(),
                    action: DialogueAction::ProvideGuidance(
                        "I'm here to help. Tell me more about what you're trying to achieve.".to_string()
                    ),
                    icon: "💡".to_string(),
                });
            }
        }
        
        // Always include these standard options
        options.push(DialogueOption {
            id: 98,
            text: "Take over the task".to_string(),
            action: DialogueAction::TakeOver,
            icon: "👨‍💻".to_string(),
        });
        
        options.push(DialogueOption {
            id: 99,
            text: "Let agent continue for now".to_string(),
            action: DialogueAction::IgnoreForNow,
            icon: "⏭️".to_string(),
        });
        
        options
    }
    
    /// Check if there are any active dialogues
    pub fn has_active_dialogues(&self) -> bool {
        !self.active_dialogues.is_empty()
    }
    
    /// Get dialogue for a specific agent
    pub fn get_dialogue(&self, agent_id: &str) -> Option<&DialogueSession> {
        self.active_dialogues.get(agent_id)
    }
    
    /// Process supervisor's choice
    pub async fn choose_option(&mut self, agent_id: String, option_id: u32) -> Option<DialogueAction> {
        if let Some(dialogue) = self.active_dialogues.get(&agent_id) {
            if let Some(option) = dialogue.options.iter().find(|o| o.id == option_id) {
                let action = option.action.clone();
                
                // Remove dialogue if it's a final action
                match action {
                    DialogueAction::TakeOver | DialogueAction::IgnoreForNow => {
                        self.active_dialogues.remove(&agent_id);
                    }
                    _ => {
                        // Update dialogue timestamp
                        if let Some(dialogue) = self.active_dialogues.get_mut(&agent_id) {
                            dialogue.last_interaction = Instant::now();
                        }
                    }
                }
                
                return Some(action);
            }
        }
        None
    }
    
    /// Auto-respond to pending dialogues (for testing)
    pub async fn tick(&mut self) {
        if !self.auto_respond {
            return;
        }
        
        // Process queued responses
        if let Some(response) = self.response_queue.pop_front() {
            self.choose_option(response.agent_id, response.chosen_option).await;
        }
        
        // Auto-respond to dialogues that have been waiting too long
        for (agent_id, dialogue) in self.active_dialogues.clone().iter() {
            if dialogue.last_interaction.elapsed().as_millis() > self.response_delay_ms as u128 {
                // Choose the first available option as default
                if let Some(first_option) = dialogue.options.first() {
                    self.choose_option(agent_id.clone(), first_option.id).await;
                }
            }
        }
    }
}

impl Default for MockSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for supervisor test scenarios
pub struct SupervisorScenarioBuilder {
    supervisor: MockSupervisor,
}

impl SupervisorScenarioBuilder {
    pub fn new() -> Self {
        Self {
            supervisor: MockSupervisor::new(),
        }
    }
    
    pub fn with_auto_respond(mut self, delay_ms: u64) -> Self {
        self.supervisor = self.supervisor.with_auto_respond(delay_ms);
        self
    }
    
    pub fn with_queued_response(mut self, agent_id: impl Into<String>, option: u32) -> Self {
        self.supervisor = self.supervisor.queue_response(agent_id, option);
        self
    }
    
    pub fn build(self) -> MockSupervisor {
        self.supervisor
    }
}

impl Default for SupervisorScenarioBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DialogueSession {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            context: DialogueContext {
                situation: String::new(),
                relevant_code: String::new(),
                error_message: None,
                task_description: String::new(),
                time_stuck: std::time::Duration::from_secs(0),
            },
            options: Vec::new(),
            created_at: Instant::now(),
            last_interaction: Instant::now(),
        }
    }
}

impl Default for SupervisorResponse {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            chosen_option: 0,
            custom_message: None,
            timestamp: Instant::now(),
        }
    }
}