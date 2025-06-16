use crate::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// MCP server implementation
#[derive(Debug)]
pub struct McpServer {
    config: McpConfig,
    protocol: McpProtocol,
    session: Arc<RwLock<ProtocolSession>>,
    resources: Arc<RwLock<HashMap<String, ResourceDefinition>>>,
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,
    prompts: Arc<RwLock<HashMap<String, PromptDefinition>>>,
    event_sender: broadcast::Sender<ServerEvent>,
}

/// Server events
#[derive(Debug, Clone)]
pub enum ServerEvent {
    ClientConnected,
    ClientDisconnected,
    ResourceUpdated(String),
    ToolCalled(String),
    PromptRequested(String),
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(config: McpConfig) -> Result<Self, McpError> {
        let protocol = McpProtocol::new(config.protocol_version.clone());
        let session = Arc::new(RwLock::new(ProtocolSession::new()));
        let (event_sender, _) = broadcast::channel(1000);
        
        Ok(Self {
            config,
            protocol,
            session,
            resources: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        })
    }
    
    /// Start the server
    pub async fn start(&mut self) -> Result<(), McpError> {
        match self.config.transport.transport_type {
            TransportType::Stdio => self.start_stdio_server().await,
            TransportType::WebSocket => self.start_websocket_server().await,
            TransportType::Tcp => self.start_tcp_server().await,
            TransportType::Unix => self.start_unix_server().await,
        }
    }
    
    /// Register a resource
    pub async fn register_resource(&self, resource: ResourceDefinition) -> Result<(), McpError> {
        let mut resources = self.resources.write().await;
        resources.insert(resource.uri.clone(), resource);
        Ok(())
    }
    
    /// Register a tool
    pub async fn register_tool(&self, tool: ToolDefinition) -> Result<(), McpError> {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), tool);
        Ok(())
    }
    
    /// Register a prompt
    pub async fn register_prompt(&self, prompt: PromptDefinition) -> Result<(), McpError> {
        let mut prompts = self.prompts.write().await;
        prompts.insert(prompt.name.clone(), prompt);
        Ok(())
    }
    
    /// Get server capabilities
    pub fn get_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            logging: Some(LoggingCapability {}),
            prompts: Some(PromptsCapability {
                list_changed: true,
            }),
            resources: Some(ResourcesCapability {
                subscribe: true,
                list_changed: true,
            }),
            tools: Some(ToolsCapability {
                list_changed: true,
            }),
        }
    }
    
    /// Get prompt by name
    pub async fn get_prompt(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<PromptResult, McpError> {
        let prompts = self.prompts.read().await;
        
        if let Some(prompt_def) = prompts.get(name) {
            // Emit event
            let _ = self.event_sender.send(ServerEvent::PromptRequested(name.to_string()));
            
            // For demonstration, return a simple prompt result
            Ok(PromptResult {
                description: prompt_def.description.clone(),
                messages: vec![
                    PromptMessage {
                        role: "user".to_string(),
                        content: PromptContent {
                            type_: "text".to_string(),
                            text: format!("Execute prompt: {}", name),
                        },
                    }
                ],
            })
        } else {
            Err(McpError::ResourceError(format!("Prompt not found: {}", name)))
        }
    }
    
    /// Subscribe to server events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ServerEvent> {
        self.event_sender.subscribe()
    }
    
    async fn start_stdio_server(&mut self) -> Result<(), McpError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::io::{stdin, stdout};
        
        tracing::info!("Starting MCP server on stdio");
        
        let stdin = stdin();
        let mut stdout = stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(response) = self.handle_message_str(&line).await {
                        if let Some(response_msg) = response {
                            let response_str = self.protocol.serialize_message(&response_msg)?;
                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn start_websocket_server(&mut self) -> Result<(), McpError> {
        // WebSocket implementation would go here
        tracing::warn!("WebSocket transport not yet implemented");
        Ok(())
    }
    
    async fn start_tcp_server(&mut self) -> Result<(), McpError> {
        // TCP implementation would go here
        tracing::warn!("TCP transport not yet implemented");
        Ok(())
    }
    
    async fn start_unix_server(&mut self) -> Result<(), McpError> {
        // Unix socket implementation would go here
        tracing::warn!("Unix socket transport not yet implemented");
        Ok(())
    }
    
    async fn handle_message_str(&self, message_str: &str) -> Result<Option<McpMessage>, McpError> {
        let message = self.protocol.parse_message(message_str)?;
        self.handle_message(message).await
    }
    
    async fn handle_message(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        self.protocol.validate_message(&message)?;
        
        match message.method.as_deref() {
            Some("initialize") => self.handle_initialize(message).await,
            Some("resources/list") => self.handle_list_resources(message).await,
            Some("resources/read") => self.handle_read_resource(message).await,
            Some("tools/list") => self.handle_list_tools(message).await,
            Some("tools/call") => self.handle_call_tool(message).await,
            Some("prompts/list") => self.handle_list_prompts(message).await,
            Some("prompts/get") => self.handle_get_prompt_message(message).await,
            Some(method) => {
                if let Some(id) = message.id {
                    Ok(Some(self.protocol.create_error_response(
                        Some(id),
                        McpRpcError::method_not_found(method),
                    )))
                } else {
                    // Notification - no response needed
                    Ok(None)
                }
            }
            None => {
                // This is a response or notification, not a request
                Ok(None)
            }
        }
    }
    
    async fn handle_initialize(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        let mut session = self.session.write().await;
        session.set_state(ProtocolState::Initializing);
        
        if let Some(params) = message.params {
            session.set_client_info(params.clone());
        }
        
        let capabilities = self.get_capabilities();
        session.set_capabilities(capabilities.clone());
        session.set_state(ProtocolState::Initialized);
        
        let _ = self.event_sender.send(ServerEvent::ClientConnected);
        
        let result = serde_json::json!({
            "protocolVersion": self.config.protocol_version,
            "capabilities": capabilities,
            "serverInfo": {
                "name": self.config.server_name,
                "version": self.config.server_version
            }
        });
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_success_response(id, result)))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_list_resources(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        let resources = self.resources.read().await;
        let resource_list: Vec<_> = resources.values().cloned().collect();
        
        let result = serde_json::json!({
            "resources": resource_list
        });
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_success_response(id, result)))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_read_resource(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        if let Some(params) = message.params {
            if let Some(uri) = params.get("uri").and_then(|u| u.as_str()) {
                let resources = self.resources.read().await;
                
                if let Some(resource) = resources.get(uri) {
                    // Emit event
                    let _ = self.event_sender.send(ServerEvent::ResourceUpdated(uri.to_string()));
                    
                    // For demonstration, return mock resource content
                    let result = serde_json::json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": resource.mime_type,
                            "text": format!("Content for resource: {}", resource.name)
                        }]
                    });
                    
                    if let Some(id) = message.id {
                        return Ok(Some(self.protocol.create_success_response(id, result)));
                    }
                } else {
                    if let Some(id) = message.id {
                        return Ok(Some(self.protocol.create_error_response(
                            Some(id),
                            McpRpcError::custom_error(404, &format!("Resource not found: {}", uri)),
                        )));
                    }
                }
            }
        }
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_error_response(
                Some(id),
                McpRpcError::invalid_params("Missing or invalid uri parameter"),
            )))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_list_tools(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        let tools = self.tools.read().await;
        let tool_list: Vec<_> = tools.values().cloned().collect();
        
        let result = serde_json::json!({
            "tools": tool_list
        });
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_success_response(id, result)))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_call_tool(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        if let Some(params) = message.params {
            if let (Some(name), Some(arguments)) = (
                params.get("name").and_then(|n| n.as_str()),
                params.get("arguments")
            ) {
                let tools = self.tools.read().await;
                
                if tools.contains_key(name) {
                    // Emit event
                    let _ = self.event_sender.send(ServerEvent::ToolCalled(name.to_string()));
                    
                    // For demonstration, return mock tool result
                    let result = serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Tool {} executed with arguments: {}", name, arguments)
                        }],
                        "isError": false
                    });
                    
                    if let Some(id) = message.id {
                        return Ok(Some(self.protocol.create_success_response(id, result)));
                    }
                } else {
                    if let Some(id) = message.id {
                        return Ok(Some(self.protocol.create_error_response(
                            Some(id),
                            McpRpcError::custom_error(404, &format!("Tool not found: {}", name)),
                        )));
                    }
                }
            }
        }
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_error_response(
                Some(id),
                McpRpcError::invalid_params("Missing or invalid tool parameters"),
            )))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_list_prompts(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        let prompts = self.prompts.read().await;
        let prompt_list: Vec<_> = prompts.values().cloned().collect();
        
        let result = serde_json::json!({
            "prompts": prompt_list
        });
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_success_response(id, result)))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_get_prompt_message(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError> {
        if let Some(params) = message.params {
            if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                let arguments = params.get("arguments");
                
                match self.get_prompt(name, arguments.cloned()).await {
                    Ok(prompt_result) => {
                        let result = serde_json::to_value(prompt_result)?;
                        
                        if let Some(id) = message.id {
                            return Ok(Some(self.protocol.create_success_response(id, result)));
                        }
                    }
                    Err(e) => {
                        if let Some(id) = message.id {
                            return Ok(Some(self.protocol.create_error_response(
                                Some(id),
                                McpRpcError::custom_error(404, &e.to_string()),
                            )));
                        }
                    }
                }
            }
        }
        
        if let Some(id) = message.id {
            Ok(Some(self.protocol.create_error_response(
                Some(id),
                McpRpcError::invalid_params("Missing or invalid prompt parameters"),
            )))
        } else {
            Ok(None)
        }
    }
}