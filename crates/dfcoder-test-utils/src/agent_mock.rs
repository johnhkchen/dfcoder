use std::time::{Duration, Instant};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use dfcoder_core::{AgentStatus, AgentMetrics};

/// Mock agent for testing scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MockAgent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub work_duration: Duration,
    #[serde(skip)]
    pub created_at: Instant,
    #[serde(skip)]
    pub last_activity: Instant,
    pub help_requests: VecDeque<HelpRequest>,
    pub responses: VecDeque<String>,
    pub metrics: AgentMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HelpRequest {
    pub message: String,
    pub context: String,
    #[serde(skip)]
    pub timestamp: Instant,
    pub urgency: HelpUrgency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HelpUrgency {
    Low,
    Medium,
    High,
    Critical,
}

impl MockAgent {
    /// Create a new mock agent
    pub fn new(name: impl Into<String>) -> Self {
        let now = Instant::now();
        let name = name.into();
        Self {
            id: format!("mock_{}", name),
            name,
            status: AgentStatus::Idle,
            current_task: None,
            work_duration: Duration::from_secs(0),
            created_at: now,
            last_activity: now,
            help_requests: VecDeque::new(),
            responses: VecDeque::new(),
            metrics: AgentMetrics::default(),
        }
    }
    
    /// Set the agent's current task
    pub fn with_current_task(mut self, task: impl Into<String>) -> Self {
        self.current_task = Some(task.into());
        self.status = AgentStatus::Working;
        self
    }
    
    /// Set how long the agent has been working
    pub fn working_for(mut self, duration: Duration) -> Self {
        self.work_duration = duration;
        self.last_activity = Instant::now() - duration;
        self
    }
    
    /// Set the agent's status
    pub fn with_status(mut self, status: AgentStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Make the agent request help
    pub fn requesting_help(mut self, message: impl Into<String>) -> Self {
        self.help_requests.push_back(HelpRequest {
            message: message.into(),
            context: self.current_task.clone().unwrap_or_default(),
            timestamp: Instant::now(),
            urgency: HelpUrgency::Medium,
        });
        self.status = AgentStatus::NeedsSupervision;
        self
    }
    
    /// Add a pre-programmed response
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.responses.push_back(response.into());
        self
    }
    
    /// Simulate the agent making progress
    pub async fn make_progress(&mut self) {
        self.last_activity = Instant::now();
        self.metrics.tasks_completed += 1;
        if self.status == AgentStatus::Stuck {
            self.status = AgentStatus::Working;
        }
    }
    
    /// Simulate the agent getting stuck
    pub async fn get_stuck(&mut self, reason: impl Into<String>) {
        self.status = AgentStatus::Stuck;
        self.help_requests.push_back(HelpRequest {
            message: format!("Stuck: {}", reason.into()),
            context: self.current_task.clone().unwrap_or_default(),
            timestamp: Instant::now(),
            urgency: HelpUrgency::High,
        });
    }
    
    /// Simulate receiving supervision
    pub async fn receive_supervision(&mut self, guidance: impl Into<String>) {
        if let Some(_help_request) = self.help_requests.pop_front() {
            self.responses.push_back(format!("Received: {}", guidance.into()));
            self.status = AgentStatus::Working;
            self.last_activity = Instant::now();
        }
    }
    
    /// Check if agent needs help
    pub fn needs_help(&self) -> bool {
        !self.help_requests.is_empty() || 
        matches!(self.status, AgentStatus::NeedsSupervision | AgentStatus::Stuck)
    }
    
    /// Get the next help request
    pub fn next_help_request(&mut self) -> Option<HelpRequest> {
        self.help_requests.pop_front()
    }
    
    /// Simulate time passing and check for timeout conditions
    pub async fn tick(&mut self, elapsed: Duration) {
        // If working too long without progress, get stuck
        if matches!(self.status, AgentStatus::Working) {
            if self.last_activity.elapsed() > Duration::from_secs(300) { // 5 minutes
                self.get_stuck("No progress for too long").await;
            }
        }
        
        // Update metrics
        if matches!(self.status, AgentStatus::Working) {
            self.metrics.response_time_ms = elapsed.as_millis() as u64;
        }
    }
}

/// Builder for complex agent scenarios
pub struct AgentScenarioBuilder {
    agent: MockAgent,
}

impl AgentScenarioBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            agent: MockAgent::new(name),
        }
    }
    
    pub fn working_on(mut self, task: impl Into<String>) -> Self {
        self.agent = self.agent.with_current_task(task);
        self
    }
    
    pub fn for_duration(mut self, duration: Duration) -> Self {
        self.agent = self.agent.working_for(duration);
        self
    }
    
    pub fn with_help_request(mut self, message: impl Into<String>) -> Self {
        self.agent = self.agent.requesting_help(message);
        self
    }
    
    pub fn that_is_stuck(mut self) -> Self {
        self.agent.status = AgentStatus::Stuck;
        self
    }
    
    pub fn build(self) -> MockAgent {
        self.agent
    }
}

impl Default for MockAgent {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            id: String::new(),
            name: String::new(),
            status: AgentStatus::default(),
            current_task: None,
            work_duration: Duration::from_secs(0),
            created_at: now,
            last_activity: now,
            help_requests: VecDeque::new(),
            responses: VecDeque::new(),
            metrics: AgentMetrics::default(),
        }
    }
}

impl Default for HelpRequest {
    fn default() -> Self {
        Self {
            message: String::new(),
            context: String::new(),
            timestamp: Instant::now(),
            urgency: HelpUrgency::Low,
        }
    }
}

// AgentMetrics default implementation is in dfcoder-types