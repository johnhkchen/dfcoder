//! BAML-based activity classification for agent output

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Classification result for agent activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityClass {
    /// Primary activity category
    pub primary: ActivityType,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
    /// Whether the agent needs help based on low confidence
    pub needs_help: bool,
    /// Detected emotional state of the agent
    pub emotional_state: EmotionalState,
    /// Estimated time until completion
    pub estimated_completion: Option<Duration>,
}

/// Types of activities an agent can be performing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    /// Setting up project structure
    Scaffolding,
    /// Writing new code/features
    Implementing,
    /// Analyzing and fixing errors
    Debugging,
    /// Writing or running tests
    Testing,
    /// Reading documentation or code
    Researching,
    /// Waiting for user input or external resources
    Waiting,
    /// Stuck and unable to proceed
    Stuck,
    /// Taking a break or idle
    Idle,
}

/// Emotional/confidence state of the agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionalState {
    /// Making good progress, confident
    Confident,
    /// Working steadily, moderate confidence
    Focused,
    /// Encountering some difficulties
    Cautious,
    /// Struggling with the task
    Frustrated,
    /// Completely stuck, needs help
    Desperate,
}

/// Errors that can occur during classification
#[derive(Debug, Error)]
pub enum ClassificationError {
    #[error("BAML API error: {0}")]
    ApiError(String),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Authentication error")]
    AuthError,
    #[error("Rate limit exceeded")]
    RateLimitError,
}

/// BAML client for activity classification
#[derive(Debug, Clone)]
pub struct ActivityClassifier {
    api_key: String,
    base_url: String,
    model: String,
}

impl ActivityClassifier {
    /// Create a new classifier with API configuration
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.baml.ai/v1".to_string(),
            model: "gpt-4".to_string(), // Default model
        }
    }

    /// Create classifier with custom configuration
    pub fn with_config(api_key: String, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
        }
    }

    /// Classify agent activity from recent output
    pub async fn classify_activity(&self, output: &str) -> Result<ActivityClass, ClassificationError> {
        self.classify_with_context(output, None).await
    }

    /// Classify activity with additional context
    pub async fn classify_with_context(
        &self,
        output: &str,
        context: Option<&ActivityContext>,
    ) -> Result<ActivityClass, ClassificationError> {
        let prompt = self.build_classification_prompt(output, context);
        
        // For now, implement a simple rule-based classifier
        // In a real implementation, this would call the BAML API
        Ok(self.rule_based_classify(output))
    }

    /// Build the prompt for BAML classification
    fn build_classification_prompt(&self, output: &str, context: Option<&ActivityContext>) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("Classify the following agent output into activity categories:\n\n");
        prompt.push_str(&format!("Output: {}\n\n", output));
        
        if let Some(ctx) = context {
            prompt.push_str(&format!("Recent history: {:?}\n", ctx.recent_activities));
            prompt.push_str(&format!("Time working: {:?}\n", ctx.time_working));
            prompt.push_str(&format!("Last known task: {:?}\n", ctx.current_task));
        }
        
        prompt.push_str("
Categories:
- Scaffolding: Setting up project structure, creating directories, config files
- Implementing: Writing new code, adding features, building functionality  
- Debugging: Analyzing errors, fixing bugs, troubleshooting issues
- Testing: Writing tests, running test suites, validating functionality
- Researching: Reading docs, studying code, learning about libraries
- Waiting: Waiting for input, external resources, or user decisions
- Stuck: Unable to proceed, needs guidance or help
- Idle: Taking a break, no active work

Emotional States:
- Confident: Making good progress, clear direction
- Focused: Working steadily, moderate confidence
- Cautious: Encountering some difficulties but proceeding
- Frustrated: Struggling significantly with the task
- Desperate: Completely stuck, needs immediate help

Please respond with JSON in this format:
{
  \"primary\": \"ActivityType\",
  \"confidence\": 0.85,
  \"emotional_state\": \"EmotionalState\",
  \"estimated_completion\": \"5m\"
}
");
        
        prompt
    }

    /// Simple rule-based classifier for demonstration
    fn rule_based_classify(&self, output: &str) -> ActivityClass {
        let output_lower = output.to_lowercase();
        
        // Detect activity type based on keywords
        let primary = if output_lower.contains("error") || output_lower.contains("failed") || output_lower.contains("exception") {
            if output_lower.contains("fixing") || output_lower.contains("debug") {
                ActivityType::Debugging
            } else {
                ActivityType::Stuck
            }
        } else if output_lower.contains("test") || output_lower.contains("spec") || output_lower.contains("assert") {
            ActivityType::Testing
        } else if output_lower.contains("mkdir") || output_lower.contains("cargo init") || output_lower.contains("setup") {
            ActivityType::Scaffolding
        } else if output_lower.contains("implementing") || output_lower.contains("writing") || output_lower.contains("adding") {
            ActivityType::Implementing
        } else if output_lower.contains("reading") || output_lower.contains("docs") || output_lower.contains("researching") {
            ActivityType::Researching
        } else if output_lower.contains("waiting") || output_lower.contains("pending") {
            ActivityType::Waiting
        } else if output_lower.contains("stuck") || output_lower.contains("confused") || output_lower.contains("help") {
            ActivityType::Stuck
        } else {
            ActivityType::Implementing // Default
        };

        // Determine confidence based on emotional indicators
        let (confidence, emotional_state) = if output_lower.contains("stuck") && (output_lower.contains("confused") || output_lower.contains("help")) {
            (0.1, EmotionalState::Desperate)
        } else if output_lower.contains("error") || output_lower.contains("failed") {
            if output_lower.contains("stuck") {
                (0.2, EmotionalState::Frustrated)
            } else {
                (0.4, EmotionalState::Frustrated)
            }
        } else if output_lower.contains("trying") || output_lower.contains("attempting") {
            (0.6, EmotionalState::Cautious)
        } else if output_lower.contains("completed") || output_lower.contains("success") || output_lower.contains("done") {
            (0.9, EmotionalState::Confident)
        } else {
            (0.7, EmotionalState::Focused)
        };

        // Determine if help is needed - only for really stuck situations
        let needs_help = matches!(primary, ActivityType::Stuck) || 
                        (confidence < 0.3 && matches!(emotional_state, EmotionalState::Desperate));

        ActivityClass {
            primary,
            confidence,
            needs_help,
            emotional_state,
            estimated_completion: Some(Duration::from_secs(300)), // Default 5 minutes
        }
    }
}

/// Context information for activity classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityContext {
    /// Recent activity history
    pub recent_activities: Vec<ActivityType>,
    /// How long the agent has been working on current task
    #[serde(skip)]
    pub time_working: Duration,
    /// Current task description
    pub current_task: Option<String>,
    /// Number of recent errors
    pub error_count: u32,
    /// Agent's role
    pub agent_role: Option<String>,
}

impl ActivityContext {
    /// Create new context
    pub fn new() -> Self {
        Self {
            recent_activities: Vec::new(),
            time_working: Duration::from_secs(0),
            current_task: None,
            error_count: 0,
            agent_role: None,
        }
    }

    /// Add an activity to the history
    pub fn add_activity(&mut self, activity: ActivityType) {
        self.recent_activities.push(activity);
        
        // Keep only last 10 activities
        if self.recent_activities.len() > 10 {
            self.recent_activities.remove(0);
        }
    }

    /// Update working time
    pub fn update_working_time(&mut self, time: Duration) {
        self.time_working = time;
    }

    /// Increment error count
    pub fn increment_errors(&mut self) {
        self.error_count += 1;
    }
}

impl Default for ActivityContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple function to classify agent output - main API
pub async fn classify_activity(output: &str) -> ActivityClass {
    let classifier = ActivityClassifier::new("demo-key".to_string());
    classifier.rule_based_classify(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_based_classification() {
        let classifier = ActivityClassifier::new("test-key".to_string());

        // Test error detection
        let result = classifier.rule_based_classify("Error: compilation failed");
        assert_eq!(result.primary, ActivityType::Stuck);
        assert!(result.needs_help);

        // Test debugging detection
        let result = classifier.rule_based_classify("Debugging the error, fixing the issue");
        assert_eq!(result.primary, ActivityType::Debugging);

        // Test test detection
        let result = classifier.rule_based_classify("Running tests to verify functionality");
        assert_eq!(result.primary, ActivityType::Testing);

        // Test scaffolding detection
        let result = classifier.rule_based_classify("Setting up project structure with cargo init");
        assert_eq!(result.primary, ActivityType::Scaffolding);

        // Test confidence levels
        let result = classifier.rule_based_classify("Successfully completed the implementation");
        assert_eq!(result.emotional_state, EmotionalState::Confident);
        assert!(!result.needs_help);
    }

    #[test]
    fn test_activity_context() {
        let mut context = ActivityContext::new();
        
        context.add_activity(ActivityType::Implementing);
        context.add_activity(ActivityType::Testing);
        
        assert_eq!(context.recent_activities.len(), 2);
        assert_eq!(context.recent_activities[0], ActivityType::Implementing);
        assert_eq!(context.recent_activities[1], ActivityType::Testing);
    }

    #[test]
    fn test_confidence_thresholds() {
        let classifier = ActivityClassifier::new("test-key".to_string());

        let high_confidence = classifier.rule_based_classify("Everything working perfectly");
        assert!(high_confidence.confidence > 0.8);

        let low_confidence = classifier.rule_based_classify("I'm stuck and need help");
        assert!(low_confidence.confidence < 0.5);
        assert!(low_confidence.needs_help);
    }

    #[tokio::test]
    async fn test_classify_activity_function() {
        let result = classify_activity("Error: cannot find module").await;
        assert_eq!(result.primary, ActivityType::Stuck);
        assert!(result.needs_help);
    }
}