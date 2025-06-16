//! MCP protocol bridge for universal tool interoperability
//! 
//! This crate implements the Model Context Protocol (MCP) for DFCoder,
//! enabling seamless integration with external tools and services
//! through a standardized protocol.

use dfcoder_macros::mcp_resources;
use dfcoder_types::*;
use dfcoder_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use client::*;
pub use server::*;
pub use resources::*;
pub use protocol::*;
pub use transport::*;

mod client;
mod server;
mod resources;
mod protocol;
mod transport;

// Define resource exposures using the DSL
mcp_resources! {
    resource agents {
        list: active_agents with status,
        read: agent_state(id: AgentId),
        write: send_command(id: AgentId, cmd: Command);
    }
}

/// MCP configuration for DFCoder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server name for identification
    pub server_name: String,
    /// Server version
    pub server_version: String,
    /// Supported MCP protocol version
    pub protocol_version: String,
    /// Transport configuration
    pub transport: TransportConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Resource configuration
    pub resources: ResourceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport type (stdio, websocket, tcp)
    pub transport_type: TransportType,
    /// Address for network transports
    pub address: Option<String>,
    /// Port for network transports
    pub port: Option<u16>,
    /// Connection timeout
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportType {
    Stdio,
    WebSocket,
    Tcp,
    Unix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub require_auth: bool,
    /// API keys for authentication
    pub api_keys: Vec<String>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst size
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Enable agent resource exposure
    pub expose_agents: bool,
    /// Enable pane resource exposure
    pub expose_panes: bool,
    /// Enable task resource exposure
    pub expose_tasks: bool,
    /// Enable metrics resource exposure
    pub expose_metrics: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server_name: "dfcoder".to_string(),
            server_version: "1.0.0".to_string(),
            protocol_version: "2024-11-05".to_string(),
            transport: TransportConfig {
                transport_type: TransportType::Stdio,
                address: None,
                port: None,
                timeout_ms: 30000,
            },
            security: SecurityConfig {
                require_auth: false,
                api_keys: Vec::new(),
                rate_limit: None,
            },
            resources: ResourceConfig {
                expose_agents: true,
                expose_panes: true,
                expose_tasks: true,
                expose_metrics: false,
            },
        }
    }
}

/// Main MCP service for DFCoder
#[derive(Debug)]
pub struct McpService {
    config: McpConfig,
    server: McpServer,
    client: Option<McpClient>,
    resource_manager: ResourceManager,
}

impl McpService {
    /// Create a new MCP service
    pub fn new(config: McpConfig) -> Result<Self, McpError> {
        let server = McpServer::new(config.clone())?;
        let resource_manager = ResourceManager::new(config.resources.clone());
        
        Ok(Self {
            config,
            server,
            client: None,
            resource_manager,
        })
    }
    
    /// Start the MCP server
    pub async fn start_server(&mut self) -> Result<(), McpError> {
        // Register resources with the server
        self.register_resources().await?;
        
        // Start the server
        self.server.start().await?;
        
        tracing::info!("MCP server started on {:?}", self.config.transport.transport_type);
        Ok(())
    }
    
    /// Connect to an external MCP server as a client
    pub async fn connect_to_server(&mut self, server_url: &str) -> Result<(), McpError> {
        let client = McpClient::new(server_url).await?;
        self.client = Some(client);
        
        tracing::info!("Connected to MCP server: {}", server_url);
        Ok(())
    }
    
    /// List available agents as MCP resources
    pub async fn list_agent_resources(&self) -> Result<Vec<Resource>, McpError> {
        self.resource_manager.list_agents().await
    }
    
    /// Get agent state as MCP resource
    pub async fn get_agent_resource(&self, agent_id: &str) -> Result<Resource, McpError> {
        self.resource_manager.get_agent(agent_id).await
    }
    
    /// Send command to agent via MCP
    pub async fn send_agent_command(&self, agent_id: &str, command: AgentCommand) -> Result<CommandResult, McpError> {
        self.resource_manager.send_command(agent_id, command).await
    }
    
    /// Register a new tool via MCP
    pub async fn register_tool(&mut self, tool: ToolDefinition) -> Result<(), McpError> {
        self.server.register_tool(tool).await
    }
    
    /// Call a tool via MCP
    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<ToolResult, McpError> {
        if let Some(ref client) = self.client {
            client.call_tool(tool_name, arguments).await
        } else {
            Err(McpError::ClientNotConnected)
        }
    }
    
    /// Register prompt templates
    pub async fn register_prompt(&mut self, prompt: PromptDefinition) -> Result<(), McpError> {
        self.server.register_prompt(prompt).await
    }
    
    /// Get prompt template
    pub async fn get_prompt(&self, prompt_name: &str, arguments: Option<serde_json::Value>) -> Result<PromptResult, McpError> {
        if let Some(ref client) = self.client {
            client.get_prompt(prompt_name, arguments).await
        } else {
            self.server.get_prompt(prompt_name, arguments).await
        }
    }
    
    /// Subscribe to resource changes
    pub async fn subscribe_to_resources(&self, pattern: &str) -> Result<ResourceSubscription, McpError> {
        self.resource_manager.subscribe(pattern).await
    }
    
    /// Get server capabilities
    pub fn get_capabilities(&self) -> ServerCapabilities {
        self.server.get_capabilities()
    }
    
    async fn register_resources(&mut self) -> Result<(), McpError> {
        if self.config.resources.expose_agents {
            let agent_resource = ResourceDefinition {
                uri: "dfcoder://agents".to_string(),
                name: "DFCoder Agents".to_string(),
                description: "Active AI agents in DFCoder".to_string(),
                mime_type: "application/json".to_string(),
                capabilities: vec![
                    ResourceCapability::Read,
                    ResourceCapability::List,
                    ResourceCapability::Write,
                ],
            };
            self.server.register_resource(agent_resource).await?;
        }
        
        if self.config.resources.expose_panes {
            let pane_resource = ResourceDefinition {
                uri: "dfcoder://panes".to_string(),
                name: "DFCoder Panes".to_string(),
                description: "Terminal panes managed by DFCoder".to_string(),
                mime_type: "application/json".to_string(),
                capabilities: vec![
                    ResourceCapability::Read,
                    ResourceCapability::List,
                ],
            };
            self.server.register_resource(pane_resource).await?;
        }
        
        if self.config.resources.expose_tasks {
            let task_resource = ResourceDefinition {
                uri: "dfcoder://tasks".to_string(),
                name: "DFCoder Tasks".to_string(),
                description: "Active and completed tasks".to_string(),
                mime_type: "application/json".to_string(),
                capabilities: vec![
                    ResourceCapability::Read,
                    ResourceCapability::List,
                    ResourceCapability::Write,
                ],
            };
            self.server.register_resource(task_resource).await?;
        }
        
        Ok(())
    }
}

/// Agent command via MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCommand {
    /// Start a new task
    StartTask { task_description: String },
    /// Stop current task
    StopTask,
    /// Request status update
    GetStatus,
    /// Send message to agent
    SendMessage { message: String },
    /// Request supervision
    RequestSupervision { context: String },
}

/// Result of agent command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    pub type_: String,
    pub text: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDefinition {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Prompt execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    pub description: String,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContent {
    pub type_: String,
    pub text: String,
}

/// Resource subscription for change notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSubscription {
    pub id: String,
    pub pattern: String,
    pub active: bool,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub logging: Option<LoggingCapability>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: bool,
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: bool,
}

/// MCP error types
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Transport error: {0}")]
    TransportError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Resource error: {0}")]
    ResourceError(String),
    #[error("Tool error: {0}")]
    ToolError(String),
    #[error("Client not connected")]
    ClientNotConnected,
    #[error("Server not started")]
    ServerNotStarted,
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Builder for MCP service configuration
pub struct McpServiceBuilder {
    config: McpConfig,
}

impl McpServiceBuilder {
    pub fn new() -> Self {
        Self {
            config: McpConfig::default(),
        }
    }
    
    pub fn server_name(mut self, name: impl Into<String>) -> Self {
        self.config.server_name = name.into();
        self
    }
    
    pub fn transport(mut self, transport: TransportConfig) -> Self {
        self.config.transport = transport;
        self
    }
    
    pub fn require_auth(mut self, require: bool) -> Self {
        self.config.security.require_auth = require;
        self
    }
    
    pub fn api_keys(mut self, keys: Vec<String>) -> Self {
        self.config.security.api_keys = keys;
        self
    }
    
    pub fn expose_agents(mut self, expose: bool) -> Self {
        self.config.resources.expose_agents = expose;
        self
    }
    
    pub fn expose_panes(mut self, expose: bool) -> Self {
        self.config.resources.expose_panes = expose;
        self
    }
    
    pub fn build(self) -> Result<McpService, McpError> {
        McpService::new(self.config)
    }
}

impl Default for McpServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}