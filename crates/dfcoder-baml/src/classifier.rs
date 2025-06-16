use crate::*;
use std::collections::HashMap;

/// Activity classifier for semantic categorization
#[derive(Debug, Clone)]
pub struct ActivityClassifier {
    client: BamlClient,
    classification_cache: HashMap<String, ActivityClassification>,
}

impl ActivityClassifier {
    /// Create a new activity classifier
    pub fn new(client: BamlClient) -> Self {
        Self {
            client,
            classification_cache: HashMap::new(),
        }
    }
    
    /// Classify a single activity
    pub async fn classify(&self, context: &ActivityContext) -> Result<ActivityClassification, BamlError> {
        // Check cache first
        let cache_key = self.generate_cache_key(context);
        if let Some(cached) = self.classification_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        let prompt = self.build_activity_classification_prompt(context);
        let response = self.client.generate_response(&prompt).await?;
        
        self.parse_activity_classification(&response)
    }
    
    /// Classify multiple activities in batch
    pub async fn classify_batch(&self, contexts: &[ActivityContext]) -> Result<Vec<ActivityClassification>, BamlError> {
        let mut results = Vec::new();
        
        for context in contexts {
            let classification = self.classify(context).await?;
            results.push(classification);
        }
        
        Ok(results)
    }
    
    /// Classify agent behavior patterns
    pub async fn classify_behavior_pattern(&self, agent_activities: &[ActivityContext]) -> Result<BehaviorPatternClassification, BamlError> {
        let prompt = self.build_behavior_pattern_prompt(agent_activities);
        let response = self.client.generate_response(&prompt).await?;
        
        self.parse_behavior_pattern_classification(&response)
    }
    
    /// Detect when an agent needs supervision
    pub async fn detect_supervision_need(&self, context: &ActivityContext) -> Result<SupervisionNeed, BamlError> {
        let prompt = format!(
            "Analyze this agent activity to determine if supervision is needed:\n\n\
            Activity: {}\n\
            Duration: {:?}\n\
            Last Output: {}\n\
            Error Messages: {:?}\n\
            Progress Indicators: {:?}\n\n\
            Return JSON with:\n\
            - needs_supervision: boolean\n\
            - urgency: low/medium/high/critical\n\
            - reasoning: explanation\n\
            - suggested_intervention: specific action to take\n\
            - estimated_resolution_time: minutes",
            context.description,
            context.duration,
            context.output_text,
            context.error_messages,
            context.progress_indicators
        );
        
        let response = self.client.generate_response(&prompt).await?;
        serde_json::from_str(&response)
            .map_err(|e| BamlError::JsonError(e))
    }
    
    /// Generate contextual dialogue options for supervision
    pub async fn generate_dialogue_options(&self, context: &ActivityContext) -> Result<Vec<DialogueOption>, BamlError> {
        let prompt = format!(
            "Generate supervision dialogue options for this situation:\n\n\
            Agent Activity: {}\n\
            Current Issue: {}\n\
            Context: {}\n\n\
            Generate 3-5 dialogue options as JSON array with:\n\
            - text: the dialogue option text\n\
            - action_type: guidance/takeover/ignore/escalate\n\
            - urgency: low/medium/high\n\
            - estimated_time: minutes to complete",
            context.description,
            context.error_messages.join(", "),
            context.output_text
        );
        
        let response = self.client.generate_response(&prompt).await?;
        serde_json::from_str(&response)
            .map_err(|e| BamlError::JsonError(e))
    }
    
    fn build_activity_classification_prompt(&self, context: &ActivityContext) -> String {
        format!(
            "Classify this software development activity:\n\n\
            Activity: {}\n\
            Duration: {:?}\n\
            Output: {}\n\
            File Types: {:?}\n\
            Commands: {:?}\n\
            Error Messages: {:?}\n\n\
            Classify into one of these categories:\n\
            - CodeGeneration (creating, refactoring, testing)\n\
            - ProblemSolving (debugging, researching, analyzing)\n\
            - Collaboration (asking_for_help, explaining, reviewing)\n\n\
            Return JSON with:\n\
            - primary_category: the main category\n\
            - subcategory: specific subcategory\n\
            - confidence: 0.0 to 1.0\n\
            - indicators: keywords that influenced classification\n\
            - complexity: low/medium/high\n\
            - success_likelihood: 0.0 to 1.0",
            context.description,
            context.duration,
            context.output_text.chars().take(500).collect::<String>(),
            context.file_types,
            context.commands_executed,
            context.error_messages
        )
    }
    
    fn build_behavior_pattern_prompt(&self, activities: &[ActivityContext]) -> String {
        let activity_summary = activities.iter()
            .enumerate()
            .map(|(i, ctx)| format!("{}. {} ({})", i + 1, ctx.description, ctx.duration.as_secs()))
            .collect::<Vec<_>>()
            .join("\n");
        
        format!(
            "Analyze this sequence of agent activities to identify behavior patterns:\n\n\
            Activities:\n{}\n\n\
            Identify:\n\
            - dominant_pattern: code_focused/problem_solver/collaborative/exploratory\n\
            - efficiency: very_low/low/medium/high/very_high\n\
            - focus_areas: list of main focus areas\n\
            - collaboration_tendency: low/medium/high\n\
            - problem_solving_approach: systematic/trial_error/research_heavy\n\
            - areas_for_improvement: list of suggestions\n\n\
            Return as JSON.",
            activity_summary
        )
    }
    
    fn parse_activity_classification(&self, response: &str) -> Result<ActivityClassification, BamlError> {
        // Extract JSON from response
        let json_start = response.find('{');
        let json_end = response.rfind('}');
        
        match (json_start, json_end) {
            (Some(start), Some(end)) if start < end => {
                let json_str = &response[start..=end];
                let parsed: serde_json::Value = serde_json::from_str(json_str)?;
                
                let primary_category = self.parse_activity_category(&parsed["primary_category"])?;
                let confidence = parsed["confidence"].as_f64().unwrap_or(0.0) as f32;
                let indicators = parsed["indicators"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                
                let complexity = match parsed["complexity"].as_str().unwrap_or("medium") {
                    "low" => ActivityComplexity::Low,
                    "high" => ActivityComplexity::High,
                    _ => ActivityComplexity::Medium,
                };
                
                Ok(ActivityClassification {
                    primary_category,
                    confidence,
                    indicators,
                    complexity,
                    success_likelihood: parsed["success_likelihood"].as_f64().unwrap_or(0.5) as f32,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => Err(BamlError::ClassificationError("Invalid JSON response".to_string()))
        }
    }
    
    fn parse_activity_category(&self, value: &serde_json::Value) -> Result<Activities, BamlError> {
        match value.as_str() {
            Some("CodeGeneration") => Ok(Activities::CodeGeneration(CodeGeneration::Creating)),
            Some("ProblemSolving") => Ok(Activities::ProblemSolving(ProblemSolving::Debugging)),
            Some("Collaboration") => Ok(Activities::Collaboration(Collaboration::AskingForHelp)),
            _ => Err(BamlError::ClassificationError("Unknown activity category".to_string()))
        }
    }
    
    fn parse_behavior_pattern_classification(&self, response: &str) -> Result<BehaviorPatternClassification, BamlError> {
        let json_start = response.find('{');
        let json_end = response.rfind('}');
        
        match (json_start, json_end) {
            (Some(start), Some(end)) if start < end => {
                let json_str = &response[start..=end];
                serde_json::from_str(json_str)
                    .map_err(|e| BamlError::JsonError(e))
            }
            _ => Err(BamlError::ClassificationError("Invalid JSON response".to_string()))
        }
    }
    
    fn generate_cache_key(&self, context: &ActivityContext) -> String {
        // Simple cache key based on activity description and duration
        format!("{}_{}", context.description, context.duration.as_secs())
    }
}

/// Context for activity classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityContext {
    pub description: String,
    pub duration: Duration,
    pub output_text: String,
    pub file_types: Vec<String>,
    pub commands_executed: Vec<String>,
    pub error_messages: Vec<String>,
    pub progress_indicators: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result of activity classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityClassification {
    pub primary_category: Activities,
    pub confidence: f32,
    pub indicators: Vec<String>,
    pub complexity: ActivityComplexity,
    pub success_likelihood: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Activity complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityComplexity {
    Low,
    Medium,
    High,
}

/// Behavior pattern classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPatternClassification {
    pub dominant_pattern: String,
    pub efficiency: String,
    pub focus_areas: Vec<String>,
    pub collaboration_tendency: String,
    pub problem_solving_approach: String,
    pub areas_for_improvement: Vec<String>,
}

/// Supervision need assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionNeed {
    pub needs_supervision: bool,
    pub urgency: String,
    pub reasoning: String,
    pub suggested_intervention: String,
    pub estimated_resolution_time: u32,
}

/// Dialogue option for supervision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    pub text: String,
    pub action_type: String,
    pub urgency: String,
    pub estimated_time: u32,
}

impl ActivityContext {
    /// Create a new activity context
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            duration: Duration::from_secs(0),
            output_text: String::new(),
            file_types: Vec::new(),
            commands_executed: Vec::new(),
            error_messages: Vec::new(),
            progress_indicators: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Add output text to the context
    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.output_text = output.into();
        self
    }
    
    /// Add file types to the context
    pub fn with_file_types(mut self, file_types: Vec<String>) -> Self {
        self.file_types = file_types;
        self
    }
    
    /// Add executed commands to the context
    pub fn with_commands(mut self, commands: Vec<String>) -> Self {
        self.commands_executed = commands;
        self
    }
    
    /// Add error messages to the context
    pub fn with_errors(mut self, errors: Vec<String>) -> Self {
        self.error_messages = errors;
        self
    }
    
    /// Set the duration of the activity
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}