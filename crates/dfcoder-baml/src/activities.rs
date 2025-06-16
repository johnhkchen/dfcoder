use crate::*;
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Activity tracker for monitoring and analyzing agent behaviors
#[derive(Debug)]
pub struct ActivityTracker {
    activities: HashMap<String, Vec<TrackedActivity>>,
    classifier: ActivityClassifier,
    patterns: ActivityPatternAnalyzer,
}

/// A tracked activity with full context and classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedActivity {
    pub id: String,
    pub agent_id: String,
    pub context: ActivityContext,
    pub classification: Option<ActivityClassification>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub outcome: ActivityOutcome,
}

/// Outcome of an activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityOutcome {
    InProgress,
    Completed(CompletionDetails),
    Failed(FailureDetails),
    Abandoned(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionDetails {
    pub success_indicators: Vec<String>,
    pub artifacts_created: Vec<String>,
    pub time_to_completion: Duration,
    pub quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetails {
    pub error_messages: Vec<String>,
    pub attempted_solutions: Vec<String>,
    pub failure_reason: String,
    pub recovery_suggestions: Vec<String>,
}

impl ActivityTracker {
    /// Create a new activity tracker
    pub fn new(classifier: ActivityClassifier) -> Self {
        Self {
            activities: HashMap::new(),
            classifier,
            patterns: ActivityPatternAnalyzer::new(),
        }
    }
    
    /// Start tracking a new activity
    pub async fn start_activity(&mut self, agent_id: String, context: ActivityContext) -> Result<String, BamlError> {
        let activity_id = uuid::Uuid::new_v4().to_string();
        
        // Classify the activity
        let classification = self.classifier.classify(&context).await.ok();
        
        let tracked_activity = TrackedActivity {
            id: activity_id.clone(),
            agent_id: agent_id.clone(),
            context,
            classification,
            start_time: chrono::Utc::now(),
            end_time: None,
            outcome: ActivityOutcome::InProgress,
        };
        
        self.activities
            .entry(agent_id)
            .or_default()
            .push(tracked_activity);
        
        Ok(activity_id)
    }
    
    /// Update an ongoing activity
    pub async fn update_activity(&mut self, agent_id: &str, activity_id: &str, context: ActivityContext) -> Result<(), BamlError> {
        if let Some(activities) = self.activities.get_mut(agent_id) {
            if let Some(activity) = activities.iter_mut().find(|a| a.id == activity_id) {
                activity.context = context;
                
                // Re-classify with updated context
                if let Ok(classification) = self.classifier.classify(&activity.context).await {
                    activity.classification = Some(classification);
                }
            }
        }
        
        Ok(())
    }
    
    /// Complete an activity
    pub fn complete_activity(&mut self, agent_id: &str, activity_id: &str, completion: CompletionDetails) -> Result<(), BamlError> {
        if let Some(activities) = self.activities.get_mut(agent_id) {
            if let Some(activity) = activities.iter_mut().find(|a| a.id == activity_id) {
                activity.end_time = Some(chrono::Utc::now());
                activity.outcome = ActivityOutcome::Completed(completion);
            }
        }
        
        Ok(())
    }
    
    /// Mark an activity as failed
    pub fn fail_activity(&mut self, agent_id: &str, activity_id: &str, failure: FailureDetails) -> Result<(), BamlError> {
        if let Some(activities) = self.activities.get_mut(agent_id) {
            if let Some(activity) = activities.iter_mut().find(|a| a.id == activity_id) {
                activity.end_time = Some(chrono::Utc::now());
                activity.outcome = ActivityOutcome::Failed(failure);
            }
        }
        
        Ok(())
    }
    
    /// Get all activities for an agent
    pub fn get_agent_activities(&self, agent_id: &str) -> Option<&Vec<TrackedActivity>> {
        self.activities.get(agent_id)
    }
    
    /// Get recent activities for an agent
    pub fn get_recent_activities(&self, agent_id: &str, since: chrono::DateTime<chrono::Utc>) -> Vec<&TrackedActivity> {
        self.activities
            .get(agent_id)
            .map(|activities| {
                activities
                    .iter()
                    .filter(|a| a.start_time >= since)
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Analyze activity patterns for an agent
    pub fn analyze_patterns(&self, agent_id: &str) -> Option<ActivityPatternAnalysis> {
        self.activities
            .get(agent_id)
            .map(|activities| self.patterns.analyze(activities))
    }
    
    /// Get activity statistics
    pub fn get_statistics(&self, agent_id: &str) -> Option<ActivityStatistics> {
        self.activities
            .get(agent_id)
            .map(|activities| self.calculate_statistics(activities))
    }
    
    /// Detect productivity trends
    pub fn detect_productivity_trends(&self, agent_id: &str, window: Duration) -> Vec<ProductivityTrend> {
        if let Some(activities) = self.activities.get(agent_id) {
            self.patterns.detect_productivity_trends(activities, window)
        } else {
            Vec::new()
        }
    }
    
    fn calculate_statistics(&self, activities: &[TrackedActivity]) -> ActivityStatistics {
        let total_activities = activities.len();
        let completed_activities = activities
            .iter()
            .filter(|a| matches!(a.outcome, ActivityOutcome::Completed(_)))
            .count();
        let failed_activities = activities
            .iter()
            .filter(|a| matches!(a.outcome, ActivityOutcome::Failed(_)))
            .count();
        
        let completion_rate = if total_activities > 0 {
            completed_activities as f32 / total_activities as f32
        } else {
            0.0
        };
        
        let average_duration = activities
            .iter()
            .filter_map(|a| {
                a.end_time.map(|end| (end - a.start_time).to_std().unwrap_or_default())
            })
            .collect::<Vec<_>>()
            .iter()
            .fold(Duration::ZERO, |acc, &d| acc + d)
            .checked_div(completed_activities as u32)
            .unwrap_or_default();
        
        // Calculate category distribution
        let mut category_counts = HashMap::new();
        for activity in activities {
            if let Some(ref classification) = activity.classification {
                let category = classification.primary_category.as_str();
                *category_counts.entry(category.to_string()).or_insert(0) += 1;
            }
        }
        
        ActivityStatistics {
            total_activities,
            completed_activities,
            failed_activities,
            completion_rate,
            average_duration,
            category_distribution: category_counts,
        }
    }
}

/// Activity pattern analyzer
#[derive(Debug)]
pub struct ActivityPatternAnalyzer {
    // Pattern recognition algorithms would go here
}

impl ActivityPatternAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Analyze activity patterns
    pub fn analyze(&self, activities: &[TrackedActivity]) -> ActivityPatternAnalysis {
        let pattern_type = self.identify_pattern_type(activities);
        let efficiency_score = self.calculate_efficiency_score(activities);
        let focus_areas = self.identify_focus_areas(activities);
        let time_patterns = self.analyze_time_patterns(activities);
        
        ActivityPatternAnalysis {
            pattern_type,
            efficiency_score,
            focus_areas,
            time_patterns,
            recommendations: self.generate_recommendations(activities),
        }
    }
    
    /// Detect productivity trends over time
    pub fn detect_productivity_trends(&self, activities: &[TrackedActivity], window: Duration) -> Vec<ProductivityTrend> {
        let mut trends = Vec::new();
        let now = chrono::Utc::now();
        let window_start = now - chrono::Duration::from_std(window).unwrap();
        
        let recent_activities: Vec<_> = activities
            .iter()
            .filter(|a| a.start_time >= window_start)
            .collect();
        
        if recent_activities.len() < 3 {
            return trends;
        }
        
        // Analyze completion rate trend
        let completion_rate = recent_activities
            .iter()
            .filter(|a| matches!(a.outcome, ActivityOutcome::Completed(_)))
            .count() as f32 / recent_activities.len() as f32;
        
        if completion_rate > 0.8 {
            trends.push(ProductivityTrend::Improving);
        } else if completion_rate < 0.5 {
            trends.push(ProductivityTrend::Declining);
        } else {
            trends.push(ProductivityTrend::Stable);
        }
        
        trends
    }
    
    fn identify_pattern_type(&self, activities: &[TrackedActivity]) -> PatternType {
        let mut code_gen_count = 0;
        let mut problem_solving_count = 0;
        let mut collaboration_count = 0;
        
        for activity in activities {
            if let Some(ref classification) = activity.classification {
                match classification.primary_category {
                    Activities::CodeGeneration(_) => code_gen_count += 1,
                    Activities::ProblemSolving(_) => problem_solving_count += 1,
                    Activities::Collaboration(_) => collaboration_count += 1,
                }
            }
        }
        
        let total = code_gen_count + problem_solving_count + collaboration_count;
        if total == 0 {
            return PatternType::Unknown;
        }
        
        let code_ratio = code_gen_count as f32 / total as f32;
        let problem_ratio = problem_solving_count as f32 / total as f32;
        let collab_ratio = collaboration_count as f32 / total as f32;
        
        if code_ratio > 0.6 {
            PatternType::CodeFocused
        } else if problem_ratio > 0.5 {
            PatternType::ProblemSolver
        } else if collab_ratio > 0.4 {
            PatternType::Collaborative
        } else {
            PatternType::Balanced
        }
    }
    
    fn calculate_efficiency_score(&self, activities: &[TrackedActivity]) -> f32 {
        if activities.is_empty() {
            return 0.0;
        }
        
        let completed_count = activities
            .iter()
            .filter(|a| matches!(a.outcome, ActivityOutcome::Completed(_)))
            .count();
        
        let avg_confidence = activities
            .iter()
            .filter_map(|a| a.classification.as_ref())
            .map(|c| c.confidence)
            .sum::<f32>() / activities.len() as f32;
        
        let completion_rate = completed_count as f32 / activities.len() as f32;
        
        (completion_rate + avg_confidence) / 2.0
    }
    
    fn identify_focus_areas(&self, activities: &[TrackedActivity]) -> Vec<String> {
        let mut focus_areas = Vec::new();
        let mut file_type_counts = HashMap::new();
        
        for activity in activities {
            for file_type in &activity.context.file_types {
                *file_type_counts.entry(file_type.clone()).or_insert(0) += 1;
            }
        }
        
        // Get the most common file types as focus areas
        let mut sorted_types: Vec<_> = file_type_counts.into_iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(&a.1));
        
        for (file_type, count) in sorted_types.into_iter().take(3) {
            if count > 1 {
                focus_areas.push(file_type);
            }
        }
        
        focus_areas
    }
    
    fn analyze_time_patterns(&self, activities: &[TrackedActivity]) -> TimePatterns {
        let mut durations = Vec::new();
        
        for activity in activities {
            if let Some(end_time) = activity.end_time {
                let duration = (end_time - activity.start_time).to_std().unwrap_or_default();
                durations.push(duration);
            }
        }
        
        if durations.is_empty() {
            return TimePatterns {
                average_duration: Duration::ZERO,
                peak_productivity_hour: None,
                consistency_score: 0.0,
            };
        }
        
        let average_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        
        // Simple consistency score based on duration variance
        let avg_secs = average_duration.as_secs() as f32;
        let variance = durations
            .iter()
            .map(|d| {
                let diff = d.as_secs() as f32 - avg_secs;
                diff * diff
            })
            .sum::<f32>() / durations.len() as f32;
        
        let consistency_score = 1.0 / (1.0 + variance.sqrt() / avg_secs);
        
        TimePatterns {
            average_duration,
            peak_productivity_hour: None, // Would require more complex analysis
            consistency_score,
        }
    }
    
    fn generate_recommendations(&self, activities: &[TrackedActivity]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let failed_count = activities
            .iter()
            .filter(|a| matches!(a.outcome, ActivityOutcome::Failed(_)))
            .count();
        
        if failed_count > activities.len() / 3 {
            recommendations.push("Consider breaking down complex tasks into smaller steps".to_string());
        }
        
        let collaboration_count = activities
            .iter()
            .filter(|a| {
                a.classification
                    .as_ref()
                    .map(|c| matches!(c.primary_category, Activities::Collaboration(_)))
                    .unwrap_or(false)
            })
            .count();
        
        if collaboration_count < activities.len() / 10 {
            recommendations.push("Consider seeking more collaboration and code reviews".to_string());
        }
        
        recommendations
    }
}

impl Default for ActivityPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Results of activity pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPatternAnalysis {
    pub pattern_type: PatternType,
    pub efficiency_score: f32,
    pub focus_areas: Vec<String>,
    pub time_patterns: TimePatterns,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    CodeFocused,
    ProblemSolver,
    Collaborative,
    Balanced,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePatterns {
    pub average_duration: Duration,
    pub peak_productivity_hour: Option<u8>,
    pub consistency_score: f32,
}

/// Activity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStatistics {
    pub total_activities: usize,
    pub completed_activities: usize,
    pub failed_activities: usize,
    pub completion_rate: f32,
    pub average_duration: Duration,
    pub category_distribution: HashMap<String, usize>,
}

/// Productivity trend indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductivityTrend {
    Improving,
    Declining,
    Stable,
    Volatile,
}