use crate::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// Resource manager for MCP
#[derive(Debug)]
pub struct ResourceManager {
    config: ResourceConfig,
    agents: Arc<RwLock<HashMap<String, AgentResource>>>,
    panes: Arc<RwLock<HashMap<String, PaneResource>>>,
    tasks: Arc<RwLock<HashMap<String, TaskResource>>>,
    subscriptions: Arc<RwLock<HashMap<String, ResourceSubscription>>>,
    change_sender: broadcast::Sender<ResourceChange>,
}

/// Agent resource representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResource {
    pub id: String,
    pub name: String,
    pub status: dfcoder_core::AgentStatus,
    pub current_task: Option<String>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub metrics: AgentMetrics,
    pub capabilities: Vec<String>,
}

/// Pane resource representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneResource {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_active: bool,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub command_history: Vec<String>,
}

/// Task resource representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResource {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_agent: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub progress: f32,
}

/// Resource change event
#[derive(Debug, Clone)]
pub enum ResourceChange {
    AgentAdded(String),
    AgentUpdated(String),
    AgentRemoved(String),
    PaneAdded(String),
    PaneUpdated(String),
    PaneRemoved(String),
    TaskAdded(String),
    TaskUpdated(String),
    TaskCompleted(String),
}

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub capabilities: Vec<ResourceCapability>,
}

/// Resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceCapability {
    Read,
    Write,
    List,
    Subscribe,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(config: ResourceConfig) -> Self {
        let (change_sender, _) = broadcast::channel(1000);
        
        Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            panes: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            change_sender,
        }
    }
    
    /// List all agents as resources
    pub async fn list_agents(&self) -> Result<Vec<Resource>, McpError> {
        if !self.config.expose_agents {
            return Ok(Vec::new());
        }
        
        let agents = self.agents.read().await;
        let mut resources = Vec::new();
        
        for (id, agent) in agents.iter() {
            resources.push(Resource {
                uri: format!("dfcoder://agents/{}", id),
                name: agent.name.clone(),
                description: format!("Agent {} - Status: {:?}", agent.name, agent.status),
                mime_type: "application/json".to_string(),
            });
        }
        
        Ok(resources)
    }
    
    /// Get specific agent resource
    pub async fn get_agent(&self, agent_id: &str) -> Result<Resource, McpError> {
        if !self.config.expose_agents {
            return Err(McpError::ResourceError("Agent resources not exposed".to_string()));
        }
        
        let agents = self.agents.read().await;
        
        if let Some(agent) = agents.get(agent_id) {
            Ok(Resource {
                uri: format!("dfcoder://agents/{}", agent_id),
                name: agent.name.clone(),
                description: format!("Agent {} - Status: {:?}", agent.name, agent.status),
                mime_type: "application/json".to_string(),
            })
        } else {
            Err(McpError::ResourceError(format!("Agent not found: {}", agent_id)))
        }
    }
    
    /// Send command to agent
    pub async fn send_command(&self, agent_id: &str, command: AgentCommand) -> Result<CommandResult, McpError> {
        if !self.config.expose_agents {
            return Err(McpError::ResourceError("Agent resources not exposed".to_string()));
        }
        
        let agents = self.agents.read().await;
        
        if agents.contains_key(agent_id) {
            // In a real implementation, this would actually send the command to the agent
            // For now, we'll simulate the command execution
            match command {
                AgentCommand::StartTask { task_description } => {
                    tracing::info!("Starting task for agent {}: {}", agent_id, task_description);
                    Ok(CommandResult {
                        success: true,
                        message: format!("Task started: {}", task_description),
                        data: Some(serde_json::json!({
                            "task_id": uuid::Uuid::new_v4().to_string(),
                            "status": "started"
                        })),
                    })
                }
                AgentCommand::StopTask => {
                    tracing::info!("Stopping task for agent {}", agent_id);
                    Ok(CommandResult {
                        success: true,
                        message: "Task stopped".to_string(),
                        data: None,
                    })
                }
                AgentCommand::GetStatus => {
                    if let Some(agent) = agents.get(agent_id) {
                        Ok(CommandResult {
                            success: true,
                            message: "Status retrieved".to_string(),
                            data: Some(serde_json::to_value(agent)?),
                        })
                    } else {
                        Ok(CommandResult {
                            success: false,
                            message: "Agent not found".to_string(),
                            data: None,
                        })
                    }
                }
                AgentCommand::SendMessage { message } => {
                    tracing::info!("Sending message to agent {}: {}", agent_id, message);
                    Ok(CommandResult {
                        success: true,
                        message: "Message sent".to_string(),
                        data: None,
                    })
                }
                AgentCommand::RequestSupervision { context } => {
                    tracing::info!("Supervision requested for agent {}: {}", agent_id, context);
                    Ok(CommandResult {
                        success: true,
                        message: "Supervision request acknowledged".to_string(),
                        data: Some(serde_json::json!({
                            "supervision_id": uuid::Uuid::new_v4().to_string(),
                            "context": context
                        })),
                    })
                }
            }
        } else {
            Err(McpError::ResourceError(format!("Agent not found: {}", agent_id)))
        }
    }
    
    /// Add or update an agent resource
    pub async fn update_agent(&self, agent: AgentResource) {
        if !self.config.expose_agents {
            return;
        }
        
        let mut agents = self.agents.write().await;
        let is_new = !agents.contains_key(&agent.id);
        agents.insert(agent.id.clone(), agent.clone());
        
        let change = if is_new {
            ResourceChange::AgentAdded(agent.id.clone())
        } else {
            ResourceChange::AgentUpdated(agent.id.clone())
        };
        
        let _ = self.change_sender.send(change);
    }
    
    /// Remove an agent resource
    pub async fn remove_agent(&self, agent_id: &str) {
        if !self.config.expose_agents {
            return;
        }
        
        let mut agents = self.agents.write().await;
        if agents.remove(agent_id).is_some() {
            let _ = self.change_sender.send(ResourceChange::AgentRemoved(agent_id.to_string()));
        }
    }
    
    /// List all panes as resources
    pub async fn list_panes(&self) -> Result<Vec<Resource>, McpError> {
        if !self.config.expose_panes {
            return Ok(Vec::new());
        }
        
        let panes = self.panes.read().await;
        let mut resources = Vec::new();
        
        for (id, pane) in panes.iter() {
            resources.push(Resource {
                uri: format!("dfcoder://panes/{}", id),
                name: pane.title.clone(),
                description: format!("Pane {} - Active: {}", pane.title, pane.is_active),
                mime_type: "text/plain".to_string(),
            });
        }
        
        Ok(resources)
    }
    
    /// Get specific pane resource
    pub async fn get_pane(&self, pane_id: &str) -> Result<Resource, McpError> {
        if !self.config.expose_panes {
            return Err(McpError::ResourceError("Pane resources not exposed".to_string()));
        }
        
        let panes = self.panes.read().await;
        
        if let Some(pane) = panes.get(pane_id) {
            Ok(Resource {
                uri: format!("dfcoder://panes/{}", pane_id),
                name: pane.title.clone(),
                description: format!("Pane {} - Active: {}", pane.title, pane.is_active),
                mime_type: "text/plain".to_string(),
            })
        } else {
            Err(McpError::ResourceError(format!("Pane not found: {}", pane_id)))
        }
    }
    
    /// Add or update a pane resource
    pub async fn update_pane(&self, pane: PaneResource) {
        if !self.config.expose_panes {
            return;
        }
        
        let mut panes = self.panes.write().await;
        let is_new = !panes.contains_key(&pane.id);
        panes.insert(pane.id.clone(), pane.clone());
        
        let change = if is_new {
            ResourceChange::PaneAdded(pane.id.clone())
        } else {
            ResourceChange::PaneUpdated(pane.id.clone())
        };
        
        let _ = self.change_sender.send(change);
    }
    
    /// List all tasks as resources
    pub async fn list_tasks(&self) -> Result<Vec<Resource>, McpError> {
        if !self.config.expose_tasks {
            return Ok(Vec::new());
        }
        
        let tasks = self.tasks.read().await;
        let mut resources = Vec::new();
        
        for (id, task) in tasks.iter() {
            resources.push(Resource {
                uri: format!("dfcoder://tasks/{}", id),
                name: task.description.clone(),
                description: format!("Task {} - Status: {:?} - Progress: {:.1}%", 
                    task.description, task.status, task.progress * 100.0),
                mime_type: "application/json".to_string(),
            });
        }
        
        Ok(resources)
    }
    
    /// Add or update a task resource
    pub async fn update_task(&self, task: TaskResource) {
        if !self.config.expose_tasks {
            return;
        }
        
        let mut tasks = self.tasks.write().await;
        let is_new = !tasks.contains_key(&task.id);
        let is_completed = matches!(task.status, TaskStatus::Completed);
        
        tasks.insert(task.id.clone(), task.clone());
        
        let change = if is_completed {
            ResourceChange::TaskCompleted(task.id.clone())
        } else if is_new {
            ResourceChange::TaskAdded(task.id.clone())
        } else {
            ResourceChange::TaskUpdated(task.id.clone())
        };
        
        let _ = self.change_sender.send(change);
    }
    
    /// Subscribe to resource changes
    pub async fn subscribe(&self, pattern: &str) -> Result<ResourceSubscription, McpError> {
        let subscription_id = uuid::Uuid::new_v4().to_string();
        let subscription = ResourceSubscription {
            id: subscription_id.clone(),
            pattern: pattern.to_string(),
            active: true,
        };
        
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription.clone());
        
        Ok(subscription)
    }
    
    /// Unsubscribe from resource changes
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<(), McpError> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        Ok(())
    }
    
    /// Get resource change stream
    pub fn get_change_stream(&self) -> broadcast::Receiver<ResourceChange> {
        self.change_sender.subscribe()
    }
    
    /// Get resource content by URI
    pub async fn get_resource_content(&self, uri: &str) -> Result<String, McpError> {
        if uri.starts_with("dfcoder://agents/") {
            let agent_id = uri.strip_prefix("dfcoder://agents/").unwrap();
            let agents = self.agents.read().await;
            
            if let Some(agent) = agents.get(agent_id) {
                serde_json::to_string_pretty(agent)
                    .map_err(|e| McpError::ResourceError(format!("Failed to serialize agent: {}", e)))
            } else {
                Err(McpError::ResourceError(format!("Agent not found: {}", agent_id)))
            }
        } else if uri.starts_with("dfcoder://panes/") {
            let pane_id = uri.strip_prefix("dfcoder://panes/").unwrap();
            let panes = self.panes.read().await;
            
            if let Some(pane) = panes.get(pane_id) {
                Ok(pane.content.clone())
            } else {
                Err(McpError::ResourceError(format!("Pane not found: {}", pane_id)))
            }
        } else if uri.starts_with("dfcoder://tasks/") {
            let task_id = uri.strip_prefix("dfcoder://tasks/").unwrap();
            let tasks = self.tasks.read().await;
            
            if let Some(task) = tasks.get(task_id) {
                serde_json::to_string_pretty(task)
                    .map_err(|e| McpError::ResourceError(format!("Failed to serialize task: {}", e)))
            } else {
                Err(McpError::ResourceError(format!("Task not found: {}", task_id)))
            }
        } else {
            Err(McpError::ResourceError(format!("Unknown resource URI: {}", uri)))
        }
    }
}

/// Resource factory for creating common resource types
pub struct ResourceFactory;

impl ResourceFactory {
    /// Create an agent resource from agent state
    pub fn create_agent_resource(agent_id: String, agent_state: &AgentState) -> AgentResource {
        AgentResource {
            id: agent_id.clone(),
            name: agent_id,
            status: agent_state.status.clone(),
            current_task: agent_state.current_task.clone(),
            last_activity: agent_state.last_activity,
            metrics: agent_state.metrics.clone(),
            capabilities: vec![
                "code_generation".to_string(),
                "debugging".to_string(),
                "file_operations".to_string(),
            ],
        }
    }
    
    /// Create a pane resource from pane state
    pub fn create_pane_resource(pane_id: String, pane_state: &PaneState) -> PaneResource {
        PaneResource {
            id: pane_id,
            title: format!("Pane {}", pane_state.id),
            content: pane_state.content.clone(),
            is_active: pane_state.is_active,
            last_update: pane_state.last_update,
            command_history: Vec::new(), // Would be populated from actual pane
        }
    }
    
    /// Create a task resource
    pub fn create_task_resource(
        task_id: String,
        description: String,
        status: TaskStatus,
        assigned_agent: Option<String>,
    ) -> TaskResource {
        TaskResource {
            id: task_id,
            description,
            status,
            assigned_agent,
            created_at: chrono::Utc::now(),
            completed_at: None,
            progress: 0.0,
        }
    }
}