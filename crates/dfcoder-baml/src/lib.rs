//! BAML integration for semantic activity categorization
//! 
//! This crate provides BAML (Behavioral Analysis Markup Language) integration
//! for DFCoder, enabling semantic understanding of agent activities and
//! intelligent categorization of behaviors.

use dfcoder_macros::baml_schema;
use dfcoder_types::*;
use dfcoder_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use client::*;
pub use classifier::*;
pub use schemas::*;
pub use activities::*;

mod client;
mod classifier;
mod schemas;
mod activities;

// Define the core activity categorization schema using the DSL
baml_schema! {
    activities categorize as {
        CodeGeneration { creating, refactoring, testing },
        ProblemSolving { debugging, researching, analyzing },
        Collaboration { asking_for_help, explaining, reviewing }
    }
}

/// BAML configuration for the DFCoder system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamlConfig {
    /// API endpoint for BAML service
    pub endpoint: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Model to use for classification
    pub model: String,
    /// Temperature for response generation
    pub temperature: f32,
    /// Maximum tokens for response
    pub max_tokens: u32,
    /// Classification confidence threshold
    pub confidence_threshold: f32,
}

impl Default for BamlConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            api_key: None,
            model: "claude-3-sonnet-20240229".to_string(),
            temperature: 0.1,
            max_tokens: 1000,
            confidence_threshold: 0.7,
        }
    }
}

/// Main BAML service for activity analysis
#[derive(Debug)]
pub struct BamlService {
    client: BamlClient,
    classifier: ActivityClassifier,
    config: BamlConfig,
}

impl BamlService {
    /// Create a new BAML service
    pub fn new(config: BamlConfig) -> Result<Self, BamlError> {
        let client = BamlClient::new(config.clone())?;
        let classifier = ActivityClassifier::new(client.clone());
        
        Ok(Self {
            client,
            classifier,
            config,
        })
    }
    
    /// Classify an agent's activity based on context
    pub async fn classify_activity(&self, context: &ActivityContext) -> Result<ActivityClassification, BamlError> {
        self.classifier.classify(context).await
    }
    
    /// Analyze agent behavior patterns over time
    pub async fn analyze_behavior_patterns(&self, agent_id: &str, activities: &[ActivityContext]) -> Result<BehaviorAnalysis, BamlError> {
        let classifications = self.classifier.classify_batch(activities).await?;
        
        let mut code_generation_count = 0;
        let mut problem_solving_count = 0;
        let mut collaboration_count = 0;
        
        for classification in &classifications {
            match classification.primary_category {
                Activities::CodeGeneration(_) => code_generation_count += 1,
                Activities::ProblemSolving(_) => problem_solving_count += 1,
                Activities::Collaboration(_) => collaboration_count += 1,
            }
        }
        
        let total_activities = classifications.len() as f32;
        
        Ok(BehaviorAnalysis {
            agent_id: agent_id.to_string(),
            total_activities: classifications.len(),
            code_generation_ratio: code_generation_count as f32 / total_activities,
            problem_solving_ratio: problem_solving_count as f32 / total_activities,
            collaboration_ratio: collaboration_count as f32 / total_activities,
            dominant_pattern: self.determine_dominant_pattern(&classifications),
            efficiency_score: self.calculate_efficiency_score(&classifications),
            recommendations: self.generate_recommendations(&classifications),
        })
    }
    
    /// Generate supervision recommendations based on activity analysis
    pub async fn generate_supervision_recommendations(&self, context: &SupervisionContext) -> Result<Vec<SupervisionRecommendation>, BamlError> {
        let prompt = self.build_supervision_prompt(context);
        let response = self.client.generate_response(&prompt).await?;
        
        // Parse the response into structured recommendations
        self.parse_supervision_recommendations(&response)
    }
    
    /// Detect anomalous behavior patterns
    pub async fn detect_anomalies(&self, agent_id: &str, recent_activities: &[ActivityContext]) -> Result<Vec<BehaviorAnomaly>, BamlError> {
        let classifications = self.classifier.classify_batch(recent_activities).await?;
        let mut anomalies = Vec::new();
        
        // Detect patterns that indicate problems
        let error_rate = self.calculate_error_rate(&classifications);
        if error_rate > 0.3 {
            anomalies.push(BehaviorAnomaly {
                agent_id: agent_id.to_string(),
                anomaly_type: AnomalyType::HighErrorRate,
                severity: if error_rate > 0.5 { AnomalySeverity::High } else { AnomalySeverity::Medium },
                description: format!("High error rate detected: {:.1}%", error_rate * 100.0),
                suggested_actions: vec![
                    "Review recent changes for potential issues".to_string(),
                    "Provide additional guidance or training".to_string(),
                ],
            });
        }
        
        // Detect stuck patterns
        if self.is_stuck_pattern(&classifications) {
            anomalies.push(BehaviorAnomaly {
                agent_id: agent_id.to_string(),
                anomaly_type: AnomalyType::StuckPattern,
                severity: AnomalySeverity::Medium,
                description: "Agent appears to be stuck on repetitive tasks".to_string(),
                suggested_actions: vec![
                    "Provide supervision and guidance".to_string(),
                    "Break down task into smaller steps".to_string(),
                ],
            });
        }
        
        Ok(anomalies)
    }
    
    fn determine_dominant_pattern(&self, classifications: &[ActivityClassification]) -> BehaviorPattern {
        let mut code_gen = 0;
        let mut problem_solving = 0;
        let mut collaboration = 0;
        
        for classification in classifications {
            match classification.primary_category {
                Activities::CodeGeneration(_) => code_gen += 1,
                Activities::ProblemSolving(_) => problem_solving += 1,
                Activities::Collaboration(_) => collaboration += 1,
            }
        }
        
        if code_gen > problem_solving && code_gen > collaboration {
            BehaviorPattern::CodeFocused
        } else if problem_solving > collaboration {
            BehaviorPattern::ProblemSolver
        } else {
            BehaviorPattern::Collaborative
        }
    }
    
    fn calculate_efficiency_score(&self, classifications: &[ActivityClassification]) -> f32 {
        let mut total_confidence = 0.0;
        let mut successful_activities = 0;
        
        for classification in classifications {
            total_confidence += classification.confidence;
            if classification.confidence > self.config.confidence_threshold {
                successful_activities += 1;
            }
        }
        
        if classifications.is_empty() {
            0.0
        } else {
            (successful_activities as f32 / classifications.len() as f32) * 
            (total_confidence / classifications.len() as f32)
        }
    }
    
    fn generate_recommendations(&self, classifications: &[ActivityClassification]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let code_gen_count = classifications.iter()
            .filter(|c| matches!(c.primary_category, Activities::CodeGeneration(_)))
            .count();
        
        let collab_count = classifications.iter()
            .filter(|c| matches!(c.primary_category, Activities::Collaboration(_)))
            .count();
        
        if code_gen_count > collab_count * 3 {
            recommendations.push("Consider seeking more collaboration and code review".to_string());
        }
        
        if collab_count > code_gen_count * 2 {
            recommendations.push("Focus more on independent coding tasks".to_string());
        }
        
        let avg_confidence: f32 = classifications.iter()
            .map(|c| c.confidence)
            .sum::<f32>() / classifications.len() as f32;
        
        if avg_confidence < 0.6 {
            recommendations.push("Activities seem unclear - provide more specific guidance".to_string());
        }
        
        recommendations
    }
    
    fn build_supervision_prompt(&self, context: &SupervisionContext) -> String {
        format!(
            "Analyze this supervision context and provide recommendations:\n\n\
            Agent: {}\n\
            Current Task: {}\n\
            Issue: {}\n\
            Context: {}\n\
            Recent Activities: {:?}\n\n\
            Please provide 3-5 specific, actionable recommendations for the supervisor.",
            context.agent_id,
            context.current_task,
            context.issue_description,
            context.code_context,
            context.recent_activities
        )
    }
    
    fn parse_supervision_recommendations(&self, response: &str) -> Result<Vec<SupervisionRecommendation>, BamlError> {
        // In a real implementation, this would use more sophisticated parsing
        // For now, we'll create some example recommendations
        Ok(vec![
            SupervisionRecommendation {
                action: "Provide step-by-step guidance".to_string(),
                priority: RecommendationPriority::High,
                reasoning: "Agent appears to be stuck on complex task".to_string(),
                estimated_time: std::time::Duration::from_minutes(10),
            },
            SupervisionRecommendation {
                action: "Review code architecture".to_string(),
                priority: RecommendationPriority::Medium,
                reasoning: "Multiple errors suggest architectural issues".to_string(),
                estimated_time: std::time::Duration::from_minutes(15),
            },
        ])
    }
    
    fn calculate_error_rate(&self, classifications: &[ActivityClassification]) -> f32 {
        let error_activities = classifications.iter()
            .filter(|c| c.indicators.contains(&"error".to_string()) || 
                      c.indicators.contains(&"failed".to_string()))
            .count();
        
        if classifications.is_empty() {
            0.0
        } else {
            error_activities as f32 / classifications.len() as f32
        }
    }
    
    fn is_stuck_pattern(&self, classifications: &[ActivityClassification]) -> bool {
        // Look for repetitive problem-solving activities without progress
        let problem_solving_count = classifications.iter()
            .filter(|c| matches!(c.primary_category, Activities::ProblemSolving(_)))
            .count();
        
        problem_solving_count > classifications.len() / 2 && classifications.len() > 5
    }
}

/// Context for supervision decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionContext {
    pub agent_id: String,
    pub current_task: String,
    pub issue_description: String,
    pub code_context: String,
    pub recent_activities: Vec<ActivityContext>,
    pub error_history: Vec<String>,
}

/// Behavior analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAnalysis {
    pub agent_id: String,
    pub total_activities: usize,
    pub code_generation_ratio: f32,
    pub problem_solving_ratio: f32,
    pub collaboration_ratio: f32,
    pub dominant_pattern: BehaviorPattern,
    pub efficiency_score: f32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorPattern {
    CodeFocused,
    ProblemSolver,
    Collaborative,
    Balanced,
}

/// Supervision recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionRecommendation {
    pub action: String,
    pub priority: RecommendationPriority,
    pub reasoning: String,
    pub estimated_time: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Behavior anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAnomaly {
    pub agent_id: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    HighErrorRate,
    StuckPattern,
    UnusualActivity,
    ProductivityDrop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}