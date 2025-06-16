//! Context-aware supervision system for agent management

use crate::agents::*;
use dfcoder_baml::{classify_activity, ActivityClass, ActivityType, EmotionalState};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use thiserror::Error;

/// Supervision request generated when an agent needs help
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SupervisionRequest {
    pub agent_id: AgentId,
    pub context: String,
    pub options: Vec<SupervisionOption>,
    pub timeout: Duration,
    pub urgency: SupervisionUrgency,
    #[serde(skip)]
    pub created_at: Instant,
}

impl Default for SupervisionRequest {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            context: String::new(),
            options: Vec::new(),
            timeout: Duration::from_secs(30),
            urgency: SupervisionUrgency::Low,
            created_at: Instant::now(),
        }
    }
}

/// Available supervision options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionOption {
    pub id: u32,
    pub text: String,
    pub action: SupervisionAction,
    pub icon: String,
    pub estimated_time: Duration,
}

/// Actions that can be taken during supervision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisionAction {
    ProvideGuidance(String),
    RequestMoreInfo,
    TakeOver,
    IgnoreForNow,
    EscalateToHuman,
    BreakDownTask,
    ReassignTask(AgentRole),
    RestartAgent,
}

/// Urgency levels for supervision requests
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SupervisionUrgency {
    Low,
    Medium,
    High,
    Critical,
}

/// Errors that can occur in the supervision system
#[derive(Debug, Error)]
pub enum SupervisionError {
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),
    #[error("Invalid supervision option: {0}")]
    InvalidOption(u32),
    #[error("Supervision timeout")]
    Timeout,
    #[error("Classification error: {0}")]
    ClassificationError(String),
}

/// Context-aware supervision system
#[derive(Debug)]
pub struct SupervisionSystem {
    active_requests: HashMap<AgentId, SupervisionRequest>,
    supervision_history: Vec<SupervisionEvent>,
    stuck_threshold: Duration,
    auto_supervision: bool,
}

/// Events tracked by the supervision system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SupervisionEvent {
    pub agent_id: AgentId,
    pub event_type: SupervisionEventType,
    pub context: String,
    #[serde(skip)]
    pub timestamp: Instant,
    pub resolution: Option<SupervisionAction>,
}

impl Default for SupervisionEvent {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            event_type: SupervisionEventType::RequestGenerated,
            context: String::new(),
            timestamp: Instant::now(),
            resolution: None,
        }
    }
}

/// Types of supervision events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisionEventType {
    RequestGenerated,
    OptionSelected,
    AutoResolved,
    Timeout,
    Escalated,
}

impl SupervisionSystem {
    /// Create a new supervision system
    pub fn new() -> Self {
        Self {
            active_requests: HashMap::new(),
            supervision_history: Vec::new(),
            stuck_threshold: Duration::from_secs(300), // 5 minutes
            auto_supervision: false,
        }
    }

    /// Enable automatic supervision responses
    pub fn enable_auto_supervision(&mut self) {
        self.auto_supervision = true;
    }

    /// Set the threshold for detecting stuck agents
    pub fn set_stuck_threshold(&mut self, threshold: Duration) {
        self.stuck_threshold = threshold;
    }

    /// Check if an agent needs supervision based on activity classification
    pub async fn check_supervision_need(
        &mut self,
        agent: &Agent,
        recent_output: &str,
    ) -> Result<Option<SupervisionRequest>, SupervisionError> {
        // Skip if already has active supervision request
        if self.active_requests.contains_key(&agent.id) {
            return Ok(None);
        }

        // Classify the agent's activity
        let activity_class = classify_activity(recent_output).await;

        // Generate supervision request if needed
        if activity_class.needs_help {
            let request = self.generate_supervision_request(agent, &activity_class, recent_output)?;
            self.active_requests.insert(agent.id.clone(), request.clone());
            
            // Record event
            self.supervision_history.push(SupervisionEvent {
                agent_id: agent.id.clone(),
                event_type: SupervisionEventType::RequestGenerated,
                context: recent_output.to_string(),
                timestamp: Instant::now(),
                resolution: None,
            });

            Ok(Some(request))
        } else {
            Ok(None)
        }
    }

    /// Generate contextual supervision dialogue for an agent
    pub fn generate_supervision_dialogue(
        agent: &Agent,
        activity: &ActivityClass,
        recent_output: &str,
    ) -> Option<SupervisionRequest> {
        if activity.needs_help {
            let context = format!(
                "Agent '{}' (role: {}) seems to need help.\n\
                Activity: {:?}\n\
                Emotional state: {:?}\n\
                Confidence: {:.1}%\n\
                Recent output: {}",
                agent.id, agent.role, activity.primary, activity.emotional_state,
                activity.confidence * 100.0, recent_output
            );

            let options = generate_dialogue_options(agent, activity, recent_output);
            let urgency = determine_urgency(activity);

            Some(SupervisionRequest {
                agent_id: agent.id.clone(),
                context,
                options,
                timeout: Duration::from_secs(30),
                urgency,
                created_at: Instant::now(),
            })
        } else {
            None
        }
    }

    /// Handle a supervision response
    pub async fn handle_supervision_response(
        &mut self,
        agent_id: &AgentId,
        option_id: u32,
    ) -> Result<SupervisionAction, SupervisionError> {
        let request = self.active_requests.remove(agent_id)
            .ok_or_else(|| SupervisionError::AgentNotFound(agent_id.clone()))?;

        let option = request.options.iter()
            .find(|o| o.id == option_id)
            .ok_or_else(|| SupervisionError::InvalidOption(option_id))?;

        let action = option.action.clone();

        // Record the resolution
        self.supervision_history.push(SupervisionEvent {
            agent_id: agent_id.clone(),
            event_type: SupervisionEventType::OptionSelected,
            context: format!("Selected option {}: {}", option_id, option.text),
            timestamp: Instant::now(),
            resolution: Some(action.clone()),
        });

        Ok(action)
    }

    /// Get active supervision request for an agent
    pub fn get_active_request(&self, agent_id: &AgentId) -> Option<&SupervisionRequest> {
        self.active_requests.get(agent_id)
    }

    /// Get all active supervision requests
    pub fn get_all_active_requests(&self) -> Vec<&SupervisionRequest> {
        self.active_requests.values().collect()
    }

    /// Clean up expired supervision requests
    pub fn cleanup_expired_requests(&mut self) {
        let now = Instant::now();
        let expired_agents: Vec<_> = self.active_requests.iter()
            .filter(|(_, request)| now.duration_since(request.created_at) > request.timeout)
            .map(|(agent_id, _)| agent_id.clone())
            .collect();

        for agent_id in expired_agents {
            if let Some(_request) = self.active_requests.remove(&agent_id) {
                self.supervision_history.push(SupervisionEvent {
                    agent_id,
                    event_type: SupervisionEventType::Timeout,
                    context: "Supervision request timed out".to_string(),
                    timestamp: now,
                    resolution: None,
                });
            }
        }
    }

    /// Get supervision history for an agent
    pub fn get_agent_history(&self, agent_id: &AgentId) -> Vec<&SupervisionEvent> {
        self.supervision_history.iter()
            .filter(|event| &event.agent_id == agent_id)
            .collect()
    }

    /// Auto-resolve supervision requests if enabled
    pub async fn auto_resolve_requests(&mut self) -> Vec<(AgentId, SupervisionAction)> {
        if !self.auto_supervision {
            return Vec::new();
        }

        let mut resolutions = Vec::new();
        let agent_ids: Vec<_> = self.active_requests.keys().cloned().collect();

        for agent_id in agent_ids {
            if let Some(request) = self.active_requests.get(&agent_id) {
                // Auto-select the first safe option
                if let Some(option) = request.options.first() {
                    if matches!(option.action, SupervisionAction::ProvideGuidance(_) | SupervisionAction::RequestMoreInfo) {
                        let action = option.action.clone();
                        let option_text = option.text.clone();
                        self.active_requests.remove(&agent_id);
                        
                        self.supervision_history.push(SupervisionEvent {
                            agent_id: agent_id.clone(),
                            event_type: SupervisionEventType::AutoResolved,
                            context: format!("Auto-selected: {}", option_text),
                            timestamp: Instant::now(),
                            resolution: Some(action.clone()),
                        });

                        resolutions.push((agent_id, action));
                    }
                }
            }
        }

        resolutions
    }

    fn generate_supervision_request(
        &self,
        agent: &Agent,
        activity: &ActivityClass,
        recent_output: &str,
    ) -> Result<SupervisionRequest, SupervisionError> {
        let context = format!(
            "Agent '{}' (role: {}) needs supervision.\n\
            Current task: {:?}\n\
            Activity type: {:?}\n\
            Emotional state: {:?}\n\
            Confidence: {:.1}%\n\
            Recent output: {}",
            agent.id, agent.role, agent.current_task,
            activity.primary, activity.emotional_state,
            activity.confidence * 100.0, recent_output
        );

        let options = generate_dialogue_options(agent, activity, recent_output);
        let urgency = determine_urgency(activity);

        Ok(SupervisionRequest {
            agent_id: agent.id.clone(),
            context,
            options,
            timeout: Duration::from_secs(30),
            urgency,
            created_at: Instant::now(),
        })
    }
}

/// Generate contextual dialogue options based on the situation
pub fn generate_dialogue_options(
    _agent: &Agent,
    activity: &ActivityClass,
    recent_output: &str,
) -> Vec<SupervisionOption> {
    let mut options = Vec::new();
    let mut option_id = 1;

    // Context-specific options based on activity type
    match activity.primary {
        ActivityType::Stuck => {
            options.push(SupervisionOption {
                id: option_id,
                text: "Provide step-by-step guidance".to_string(),
                action: SupervisionAction::ProvideGuidance(
                    "Let's break this down step by step. Can you show me the exact error?".to_string()
                ),
                icon: "🔍".to_string(),
                estimated_time: Duration::from_secs(300),
            });
            option_id += 1;

            options.push(SupervisionOption {
                id: option_id,
                text: "Break down the task".to_string(),
                action: SupervisionAction::BreakDownTask,
                icon: "📝".to_string(),
                estimated_time: Duration::from_secs(180),
            });
            option_id += 1;
        }
        ActivityType::Debugging => {
            if recent_output.contains("error") {
                options.push(SupervisionOption {
                    id: option_id,
                    text: "Help analyze the error".to_string(),
                    action: SupervisionAction::ProvideGuidance(
                        "Let me help you understand this error. Here's what it means...".to_string()
                    ),
                    icon: "🛠️".to_string(),
                    estimated_time: Duration::from_secs(240),
                });
                option_id += 1;
            }
        }
        _ => {
            options.push(SupervisionOption {
                id: option_id,
                text: "Request more context".to_string(),
                action: SupervisionAction::RequestMoreInfo,
                icon: "❓".to_string(),
                estimated_time: Duration::from_secs(120),
            });
            option_id += 1;
        }
    }

    // Emotional state-specific options
    match activity.emotional_state {
        EmotionalState::Desperate => {
            options.push(SupervisionOption {
                id: option_id,
                text: "Take over the task".to_string(),
                action: SupervisionAction::TakeOver,
                icon: "👨‍💻".to_string(),
                estimated_time: Duration::from_secs(600),
            });
            option_id += 1;
        }
        EmotionalState::Frustrated => {
            options.push(SupervisionOption {
                id: option_id,
                text: "Reassign to different role".to_string(),
                action: SupervisionAction::ReassignTask(AgentRole::Debugger),
                icon: "🔄".to_string(),
                estimated_time: Duration::from_secs(60),
            });
            option_id += 1;
        }
        _ => {}
    }

    // Always include standard options
    options.push(SupervisionOption {
        id: option_id,
        text: "Let agent continue for now".to_string(),
        action: SupervisionAction::IgnoreForNow,
        icon: "⏭️".to_string(),
        estimated_time: Duration::from_secs(0),
    });
    option_id += 1;

    options.push(SupervisionOption {
        id: option_id,
        text: "Escalate to human supervisor".to_string(),
        action: SupervisionAction::EscalateToHuman,
        icon: "🚨".to_string(),
        estimated_time: Duration::from_secs(900),
    });

    options
}

/// Determine the urgency level based on the activity classification
pub fn determine_urgency(activity: &ActivityClass) -> SupervisionUrgency {
    match (activity.emotional_state.clone(), activity.confidence, activity.primary.clone()) {
        (EmotionalState::Desperate, _, _) => SupervisionUrgency::Critical,
        (EmotionalState::Frustrated, _, _) => SupervisionUrgency::High,
        (_, c, ActivityType::Stuck) if c < 0.4 => SupervisionUrgency::High,
        (_, c, _) if c <= 0.3 => SupervisionUrgency::Medium,
        _ => SupervisionUrgency::Low,
    }
}

impl Default for SupervisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentRole;

    #[tokio::test]
    async fn test_supervision_request_generation() {
        let mut supervision = SupervisionSystem::new();
        let agent = Agent::new(AgentRole::Implementer, 1);

        // Test with output that should trigger supervision
        let result = supervision.check_supervision_need(&agent, "Error: I'm stuck and need help").await;
        assert!(result.is_ok());
        
        let request = result.unwrap();
        assert!(request.is_some());
        
        let req = request.unwrap();
        assert_eq!(req.agent_id, agent.id);
        assert!(!req.options.is_empty());
    }

    #[test]
    fn test_dialogue_option_generation() {
        let agent = Agent::new(AgentRole::Debugger, 1);
        let activity = ActivityClass {
            primary: ActivityType::Stuck,
            confidence: 0.2,
            needs_help: true,
            emotional_state: EmotionalState::Frustrated,
            estimated_completion: None,
        };

        let options = generate_dialogue_options(&agent, &activity, "I'm stuck on this error");
        assert!(!options.is_empty());
        
        // Should have at least basic options
        assert!(options.iter().any(|o| matches!(o.action, SupervisionAction::ProvideGuidance(_))));
        assert!(options.iter().any(|o| matches!(o.action, SupervisionAction::IgnoreForNow)));
    }

    #[test]
    fn test_urgency_determination() {
        let desperate = ActivityClass {
            primary: ActivityType::Stuck,
            confidence: 0.1,
            needs_help: true,
            emotional_state: EmotionalState::Desperate,
            estimated_completion: None,
        };
        assert_eq!(determine_urgency(&desperate), SupervisionUrgency::Critical);

        let frustrated = ActivityClass {
            primary: ActivityType::Debugging,
            confidence: 0.2,
            needs_help: true,
            emotional_state: EmotionalState::Frustrated,
            estimated_completion: None,
        };
        assert_eq!(determine_urgency(&frustrated), SupervisionUrgency::High);

        let low_confidence = ActivityClass {
            primary: ActivityType::Implementing,
            confidence: 0.3,
            needs_help: true,
            emotional_state: EmotionalState::Cautious,
            estimated_completion: None,
        };
        assert_eq!(determine_urgency(&low_confidence), SupervisionUrgency::Medium);
    }

    #[tokio::test]
    async fn test_supervision_response_handling() {
        let mut supervision = SupervisionSystem::new();
        let agent = Agent::new(AgentRole::Implementer, 1);

        // Generate a supervision request
        let _result = supervision.check_supervision_need(&agent, "Error: stuck").await;
        
        // Handle a response
        let response = supervision.handle_supervision_response(&agent.id, 1).await;
        assert!(response.is_ok());

        // Should no longer have active request
        assert!(supervision.get_active_request(&agent.id).is_none());
    }

    #[test]
    fn test_expired_request_cleanup() {
        let mut supervision = SupervisionSystem::new();
        
        // Add a request with past timestamp
        let mut request = SupervisionRequest {
            agent_id: "test-agent".to_string(),
            context: "Test".to_string(),
            options: vec![],
            timeout: Duration::from_secs(1),
            urgency: SupervisionUrgency::Low,
            created_at: Instant::now() - Duration::from_secs(2),
        };
        
        supervision.active_requests.insert("test-agent".to_string(), request);
        
        supervision.cleanup_expired_requests();
        
        // Request should be removed
        assert!(supervision.active_requests.is_empty());
        assert!(!supervision.supervision_history.is_empty());
    }
}