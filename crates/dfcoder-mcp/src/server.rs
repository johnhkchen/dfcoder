//! MCP server implementation for DFCoder agent monitoring

use crate::protocol::*;
use dfcoder_core::{Agent, Task, WorkshopManager, AgentId, TaskId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tokio::sync::Mutex;

/// MCP server for DFCoder agent management
pub struct DFCoderMCPServer {
    agents: Arc<RwLock<HashMap<AgentId, Agent>>>,
    workshop: Arc<Mutex<WorkshopManager>>,
    event_handlers: Vec<Box<dyn Fn(McpEvent) + Send + Sync>>,
}

/// Events that can be emitted by the MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpEvent {
    AgentStateChanged {
        agent_id: AgentId,
        old_status: String,
        new_status: String,
    },
    TaskAssigned {
        task_id: TaskId,
        agent_id: AgentId,
    },
    TaskCompleted {
        task_id: TaskId,
        agent_id: AgentId,
    },
    SupervisionRequested {
        agent_id: AgentId,
        reason: String,
    },
}

/// Errors that can occur in the MCP server
#[derive(Debug, Error)]
pub enum McpServerError {
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Workshop error: {0}")]
    WorkshopError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl DFCoderMCPServer {
    /// Create a new MCP server
    pub fn new(workshop: Arc<Mutex<WorkshopManager>>) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            workshop,
            event_handlers: Vec::new(),
        }
    }

    /// Register an agent with the MCP server
    pub async fn register_agent(&self, agent: Agent) -> Result<(), McpServerError> {
        let agent_id = agent.id.clone();
        {
            let mut agents = self.agents.write().unwrap();
            agents.insert(agent_id.clone(), agent.clone());
        }

        // Register with workshop
        let mut workshop = self.workshop.lock().await;
        workshop.register_agent(agent).map_err(|e| McpServerError::WorkshopError(e.to_string()))?;

        Ok(())
    }

    /// List available resources
    pub async fn list_resources(&self) -> Vec<McpResource> {
        vec![
            McpResource {
                uri: "agents".to_string(),
                name: "Agent Management".to_string(),
                description: Some("Monitor and control DFCoder agents".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "tasks".to_string(),
                name: "Task Management".to_string(),
                description: Some("View and manage agent tasks".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "workshop".to_string(),
                name: "Workshop Status".to_string(),
                description: Some("Overall workshop capacity and metrics".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &str, params: Option<Value>) -> Result<Value, McpServerError> {
        match uri {
            "agents" => self.read_agents_resource(params).await,
            "tasks" => self.read_tasks_resource(params).await,
            "workshop" => self.read_workshop_resource().await,
            _ => Err(McpServerError::InvalidRequest(format!("Unknown resource: {}", uri))),
        }
    }

    /// Write to a resource (for controlling agents/tasks)
    pub async fn write_resource(&self, uri: &str, content: Value) -> Result<(), McpServerError> {
        match uri {
            "agents" => self.write_agents_resource(content).await,
            "tasks" => self.write_tasks_resource(content).await,
            _ => Err(McpServerError::InvalidRequest(format!("Cannot write to resource: {}", uri))),
        }
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "assign_task".to_string(),
                description: "Assign a task to an agent".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "task_id": {"type": "string"},
                        "agent_id": {"type": "string", "optional": true}
                    },
                    "required": ["task_id"]
                }),
            },
            McpTool {
                name: "stop_agent".to_string(),
                description: "Stop an agent's current task".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {"type": "string"}
                    },
                    "required": ["agent_id"]
                }),
            },
            McpTool {
                name: "get_agent_status".to_string(),
                description: "Get detailed status of an agent".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {"type": "string"}
                    },
                    "required": ["agent_id"]
                }),
            },
            McpTool {
                name: "create_task".to_string(),
                description: "Create a new task".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "role": {"type": "string", "enum": ["Scaffolder", "Implementer", "Debugger", "Tester"]},
                        "priority": {"type": "string", "enum": ["Low", "Normal", "High", "Critical"]}
                    },
                    "required": ["title", "description", "role"]
                }),
            },
        ]
    }

    /// Execute a tool
    pub async fn execute_tool(&self, name: &str, arguments: Value) -> Result<Value, McpServerError> {
        match name {
            "assign_task" => self.execute_assign_task(arguments).await,
            "stop_agent" => self.execute_stop_agent(arguments).await,
            "get_agent_status" => self.execute_get_agent_status(arguments).await,
            "create_task" => self.execute_create_task(arguments).await,
            _ => Err(McpServerError::InvalidRequest(format!("Unknown tool: {}", name))),
        }
    }

    /// List available prompts
    pub async fn list_prompts(&self) -> Vec<McpPrompt> {
        vec![
            McpPrompt {
                name: "agent_supervision".to_string(),
                description: "Generate supervision dialogue for a stuck agent".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "agent_id".to_string(),
                        description: "ID of the agent needing supervision".to_string(),
                        required: true,
                    },
                    McpPromptArgument {
                        name: "context".to_string(),
                        description: "Additional context about the situation".to_string(),
                        required: false,
                    },
                ],
            },
            McpPrompt {
                name: "task_breakdown".to_string(),
                description: "Break down a complex task into smaller subtasks".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "task_description".to_string(),
                        description: "Description of the complex task".to_string(),
                        required: true,
                    },
                    McpPromptArgument {
                        name: "target_role".to_string(),
                        description: "Target agent role for the subtasks".to_string(),
                        required: false,
                    },
                ],
            },
        ]
    }

    /// Get a prompt with arguments
    pub async fn get_prompt(&self, name: &str, arguments: Value) -> Result<McpPromptResult, McpServerError> {
        match name {
            "agent_supervision" => self.get_supervision_prompt(arguments).await,
            "task_breakdown" => self.get_task_breakdown_prompt(arguments).await,
            _ => Err(McpServerError::InvalidRequest(format!("Unknown prompt: {}", name))),
        }
    }

    // Internal resource readers
    async fn read_agents_resource(&self, params: Option<Value>) -> Result<Value, McpServerError> {
        let agents = self.agents.read().unwrap();
        
        if let Some(params) = params {
            if let Some(agent_id) = params.get("agent_id").and_then(|v| v.as_str()) {
                // Return specific agent
                if let Some(agent) = agents.get(agent_id) {
                    return Ok(serde_json::to_value(agent)?);
                } else {
                    return Err(McpServerError::AgentNotFound(agent_id.to_string()));
                }
            }
        }
        
        // Return all agents
        let agent_list: Vec<_> = agents.values().collect();
        Ok(serde_json::to_value(agent_list)?)
    }

    async fn read_tasks_resource(&self, _params: Option<Value>) -> Result<Value, McpServerError> {
        let workshop = self.workshop.lock().await;
        let queue = workshop.get_queue();
        Ok(serde_json::to_value(queue)?)
    }

    async fn read_workshop_resource(&self) -> Result<Value, McpServerError> {
        let workshop = self.workshop.lock().await;
        let status = workshop.get_status();
        Ok(serde_json::to_value(status)?)
    }

    // Internal resource writers
    async fn write_agents_resource(&self, content: Value) -> Result<(), McpServerError> {
        // Parse the action from content
        let action = content.get("action").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing action field".to_string()))?;

        match action {
            "update_status" => {
                let agent_id = content.get("agent_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpServerError::InvalidRequest("Missing agent_id".to_string()))?;
                let new_status = content.get("status").and_then(|v| v.as_str())
                    .ok_or_else(|| McpServerError::InvalidRequest("Missing status".to_string()))?;

                // Update agent status (simplified for demo)
                let mut agents = self.agents.write().unwrap();
                if let Some(agent) = agents.get_mut(agent_id) {
                    // In a real implementation, this would properly update the agent status
                    // For now, just emit an event
                    self.emit_event(McpEvent::AgentStateChanged {
                        agent_id: agent_id.to_string(),
                        old_status: format!("{:?}", agent.status),
                        new_status: new_status.to_string(),
                    });
                    Ok(())
                } else {
                    Err(McpServerError::AgentNotFound(agent_id.to_string()))
                }
            },
            _ => Err(McpServerError::InvalidRequest(format!("Unknown action: {}", action))),
        }
    }

    async fn write_tasks_resource(&self, content: Value) -> Result<(), McpServerError> {
        let action = content.get("action").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing action field".to_string()))?;

        match action {
            "create" => {
                // Parse task creation request
                let title = content.get("title").and_then(|v| v.as_str())
                    .ok_or_else(|| McpServerError::InvalidRequest("Missing title".to_string()))?;
                let description = content.get("description").and_then(|v| v.as_str())
                    .ok_or_else(|| McpServerError::InvalidRequest("Missing description".to_string()))?;
                let role_str = content.get("role").and_then(|v| v.as_str())
                    .ok_or_else(|| McpServerError::InvalidRequest("Missing role".to_string()))?;
                
                // Parse role and priority
                let role = match role_str {
                    "Scaffolder" => dfcoder_core::AgentRole::Scaffolder,
                    "Implementer" => dfcoder_core::AgentRole::Implementer,
                    "Debugger" => dfcoder_core::AgentRole::Debugger,
                    "Tester" => dfcoder_core::AgentRole::Tester,
                    _ => return Err(McpServerError::InvalidRequest("Invalid role".to_string())),
                };

                let priority = content.get("priority").and_then(|v| v.as_str())
                    .map(|p| match p {
                        "Low" => dfcoder_core::TaskPriority::Low,
                        "High" => dfcoder_core::TaskPriority::High,
                        "Critical" => dfcoder_core::TaskPriority::Critical,
                        _ => dfcoder_core::TaskPriority::Normal,
                    })
                    .unwrap_or(dfcoder_core::TaskPriority::Normal);

                // Create and queue task
                let task = Task::new(title.to_string(), description.to_string(), role, priority);
                let mut workshop = self.workshop.lock().await;
                workshop.queue_task(task);
                
                Ok(())
            },
            _ => Err(McpServerError::InvalidRequest(format!("Unknown action: {}", action))),
        }
    }

    // Tool implementations
    async fn execute_assign_task(&self, arguments: Value) -> Result<Value, McpServerError> {
        let _task_id = arguments.get("task_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing task_id".to_string()))?;

        let mut workshop = self.workshop.lock().await;
        match workshop.try_assign_next_task() {
            Ok(Some((agent_id, assigned_task_id))) => {
                self.emit_event(McpEvent::TaskAssigned {
                    task_id: assigned_task_id,
                    agent_id: agent_id.clone(),
                });
                Ok(json!({"success": true, "agent_id": agent_id}))
            },
            Ok(None) => Ok(json!({"success": false, "reason": "No available agents"})),
            Err(e) => Err(McpServerError::WorkshopError(e.to_string())),
        }
    }

    async fn execute_stop_agent(&self, arguments: Value) -> Result<Value, McpServerError> {
        let agent_id = arguments.get("agent_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing agent_id".to_string()))?;

        // In a real implementation, this would properly stop the agent
        // For now, just return success
        Ok(json!({"success": true, "agent_id": agent_id}))
    }

    async fn execute_get_agent_status(&self, arguments: Value) -> Result<Value, McpServerError> {
        let agent_id = arguments.get("agent_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing agent_id".to_string()))?;

        let agents = self.agents.read().unwrap();
        if let Some(agent) = agents.get(agent_id) {
            Ok(serde_json::to_value(agent)?)
        } else {
            Err(McpServerError::AgentNotFound(agent_id.to_string()))
        }
    }

    async fn execute_create_task(&self, arguments: Value) -> Result<Value, McpServerError> {
        self.write_tasks_resource(arguments).await?;
        Ok(json!({"success": true}))
    }

    // Prompt implementations
    async fn get_supervision_prompt(&self, arguments: Value) -> Result<McpPromptResult, McpServerError> {
        let agent_id = arguments.get("agent_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing agent_id".to_string()))?;
        
        let context = arguments.get("context").and_then(|v| v.as_str()).unwrap_or("");

        let agents = self.agents.read().unwrap();
        if let Some(agent) = agents.get(agent_id) {
            let prompt = format!(
                "The agent '{}' (role: {:?}) needs supervision. Current status: {:?}\n\
                Current task: {:?}\n\
                Additional context: {}\n\n\
                Please provide supervision guidance for this agent. Consider:\n\
                1. What specific help does the agent need?\n\
                2. Should we break down the task differently?\n\
                3. What resources or information might be missing?\n\
                4. Should we reassign this task to a different agent?\n\n\
                Provide clear, actionable guidance.",
                agent.id, agent.role, agent.status, agent.current_task, context
            );

            Ok(McpPromptResult {
                description: format!("Supervision guidance for agent {}", agent_id),
                messages: vec![
                    McpPromptMessage {
                        role: "user".to_string(),
                        content: prompt,
                    }
                ],
            })
        } else {
            Err(McpServerError::AgentNotFound(agent_id.to_string()))
        }
    }

    async fn get_task_breakdown_prompt(&self, arguments: Value) -> Result<McpPromptResult, McpServerError> {
        let task_description = arguments.get("task_description").and_then(|v| v.as_str())
            .ok_or_else(|| McpServerError::InvalidRequest("Missing task_description".to_string()))?;
        
        let target_role = arguments.get("target_role").and_then(|v| v.as_str()).unwrap_or("any");

        let prompt = format!(
            "Break down this complex task into smaller, manageable subtasks:\n\n\
            Task: {}\n\
            Target role: {}\n\n\
            Please provide:\n\
            1. A list of 3-7 specific subtasks\n\
            2. Recommended agent role for each subtask (Scaffolder/Implementer/Debugger/Tester)\n\
            3. Priority level for each subtask (Low/Normal/High/Critical)\n\
            4. Estimated time for each subtask\n\
            5. Dependencies between subtasks\n\n\
            Format the response as a structured breakdown that can be easily converted into individual tasks.",
            task_description, target_role
        );

        Ok(McpPromptResult {
            description: "Task breakdown guidance".to_string(),
            messages: vec![
                McpPromptMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
        })
    }

    // Event handling
    fn emit_event(&self, event: McpEvent) {
        for handler in &self.event_handlers {
            handler(event.clone());
        }
    }

    pub fn add_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(McpEvent) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<McpPromptArgument>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// MCP prompt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptResult {
    pub description: String,
    pub messages: Vec<McpPromptMessage>,
}

/// MCP message for prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptMessage {
    pub role: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfcoder_core::{Agent, AgentRole, WorkshopManager};

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let workshop = Arc::new(Mutex::new(WorkshopManager::new()));
        let server = DFCoderMCPServer::new(workshop);
        
        // Test basic functionality
        let resources = server.list_resources().await;
        assert!(!resources.is_empty());
        assert!(resources.iter().any(|r| r.uri == "agents"));
        assert!(resources.iter().any(|r| r.uri == "tasks"));
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let workshop = Arc::new(Mutex::new(WorkshopManager::new()));
        let server = DFCoderMCPServer::new(workshop);
        
        let agent = Agent::new(AgentRole::Implementer, 1);
        let agent_id = agent.id.clone();
        
        assert!(server.register_agent(agent).await.is_ok());
        
        // Verify agent is accessible via MCP
        let result = server.read_resource("agents", Some(json!({"agent_id": agent_id}))).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let workshop = Arc::new(Mutex::new(WorkshopManager::new()));
        let server = DFCoderMCPServer::new(workshop);
        
        // Test create_task tool
        let result = server.execute_tool(
            "create_task",
            json!({
                "title": "Test task",
                "description": "A test task",
                "role": "Implementer",
                "priority": "Normal"
            })
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prompt_generation() {
        let workshop = Arc::new(Mutex::new(WorkshopManager::new()));
        let server = DFCoderMCPServer::new(workshop);
        
        let agent = Agent::new(AgentRole::Debugger, 1);
        let agent_id = agent.id.clone();
        server.register_agent(agent).await.unwrap();
        
        let result = server.get_prompt(
            "agent_supervision",
            json!({"agent_id": agent_id, "context": "Agent is stuck on error"})
        ).await;
        
        assert!(result.is_ok());
        let prompt_result = result.unwrap();
        assert!(!prompt_result.messages.is_empty());
        assert!(prompt_result.messages[0].content.contains("supervision"));
    }
}