use crate::*;
use serde_json::Value;

/// MCP protocol implementation
#[derive(Debug, Clone)]
pub struct McpProtocol {
    version: String,
}

impl McpProtocol {
    pub fn new(version: String) -> Self {
        Self { version }
    }
    
    /// Create initialization request
    pub fn create_initialize_request(&self, capabilities: ClientCapabilities) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(self.generate_id()),
            method: Some("initialize".to_string()),
            params: Some(serde_json::json!({
                "protocolVersion": self.version,
                "capabilities": capabilities,
                "clientInfo": {
                    "name": "dfcoder",
                    "version": "1.0.0"
                }
            })),
            result: None,
            error: None,
        }
    }
    
    /// Create resource list request
    pub fn create_list_resources_request(&self) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(self.generate_id()),
            method: Some("resources/list".to_string()),
            params: None,
            result: None,
            error: None,
        }
    }
    
    /// Create resource read request
    pub fn create_read_resource_request(&self, uri: &str) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(self.generate_id()),
            method: Some("resources/read".to_string()),
            params: Some(serde_json::json!({
                "uri": uri
            })),
            result: None,
            error: None,
        }
    }
    
    /// Create tool call request
    pub fn create_tool_call_request(&self, name: &str, arguments: Value) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(self.generate_id()),
            method: Some("tools/call".to_string()),
            params: Some(serde_json::json!({
                "name": name,
                "arguments": arguments
            })),
            result: None,
            error: None,
        }
    }
    
    /// Create prompt get request
    pub fn create_get_prompt_request(&self, name: &str, arguments: Option<Value>) -> McpMessage {
        let mut params = serde_json::json!({
            "name": name
        });
        
        if let Some(args) = arguments {
            params.as_object_mut().unwrap().insert("arguments".to_string(), args);
        }
        
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(self.generate_id()),
            method: Some("prompts/get".to_string()),
            params: Some(params),
            result: None,
            error: None,
        }
    }
    
    /// Create success response
    pub fn create_success_response(&self, id: Value, result: Value) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }
    
    /// Create error response
    pub fn create_error_response(&self, id: Option<Value>, error: McpRpcError) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }
    
    /// Create notification
    pub fn create_notification(&self, method: &str, params: Option<Value>) -> McpMessage {
        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method.to_string()),
            params,
            result: None,
            error: None,
        }
    }
    
    /// Parse incoming message
    pub fn parse_message(&self, data: &str) -> Result<McpMessage, McpError> {
        serde_json::from_str(data)
            .map_err(|e| McpError::ProtocolError(format!("Failed to parse message: {}", e)))
    }
    
    /// Serialize message for transmission
    pub fn serialize_message(&self, message: &McpMessage) -> Result<String, McpError> {
        serde_json::to_string(message)
            .map_err(|e| McpError::ProtocolError(format!("Failed to serialize message: {}", e)))
    }
    
    /// Validate message format
    pub fn validate_message(&self, message: &McpMessage) -> Result<(), McpError> {
        if message.jsonrpc != "2.0" {
            return Err(McpError::ProtocolError("Invalid JSON-RPC version".to_string()));
        }
        
        // Check if it's a request, response, or notification
        match (message.method.as_ref(), message.id.as_ref(), message.result.as_ref(), message.error.as_ref()) {
            // Request: has method and id
            (Some(_), Some(_), None, None) => Ok(()),
            // Response: has id and either result or error
            (None, Some(_), Some(_), None) | (None, Some(_), None, Some(_)) => Ok(()),
            // Notification: has method but no id
            (Some(_), None, None, None) => Ok(()),
            _ => Err(McpError::ProtocolError("Invalid message format".to_string())),
        }
    }
    
    fn generate_id(&self) -> Value {
        Value::String(uuid::Uuid::new_v4().to_string())
    }
}

/// MCP JSON-RPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpRpcError>,
}

/// MCP RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpRpcError {
    pub fn invalid_request(message: &str) -> Self {
        Self {
            code: -32600,
            message: format!("Invalid Request: {}", message),
            data: None,
        }
    }
    
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }
    
    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid params: {}", message),
            data: None,
        }
    }
    
    pub fn internal_error(message: &str) -> Self {
        Self {
            code: -32603,
            message: format!("Internal error: {}", message),
            data: None,
        }
    }
    
    pub fn custom_error(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            experimental: None,
            sampling: None,
        }
    }
}

/// Message handler trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle incoming message
    async fn handle_message(&self, message: McpMessage) -> Result<Option<McpMessage>, McpError>;
    
    /// Handle initialize request
    async fn handle_initialize(&self, params: Value) -> Result<Value, McpError>;
    
    /// Handle resource list request
    async fn handle_list_resources(&self) -> Result<Value, McpError>;
    
    /// Handle resource read request
    async fn handle_read_resource(&self, uri: &str) -> Result<Value, McpError>;
    
    /// Handle tool call request
    async fn handle_tool_call(&self, name: &str, arguments: Value) -> Result<Value, McpError>;
    
    /// Handle prompt get request
    async fn handle_get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<Value, McpError>;
}

/// Protocol state machine
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolState {
    Uninitialized,
    Initializing,
    Initialized,
    Error,
}

/// Protocol session
#[derive(Debug)]
pub struct ProtocolSession {
    state: ProtocolState,
    capabilities: Option<ServerCapabilities>,
    client_info: Option<Value>,
}

impl ProtocolSession {
    pub fn new() -> Self {
        Self {
            state: ProtocolState::Uninitialized,
            capabilities: None,
            client_info: None,
        }
    }
    
    pub fn state(&self) -> &ProtocolState {
        &self.state
    }
    
    pub fn set_state(&mut self, state: ProtocolState) {
        self.state = state;
    }
    
    pub fn set_capabilities(&mut self, capabilities: ServerCapabilities) {
        self.capabilities = Some(capabilities);
    }
    
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }
    
    pub fn set_client_info(&mut self, info: Value) {
        self.client_info = Some(info);
    }
    
    pub fn is_initialized(&self) -> bool {
        self.state == ProtocolState::Initialized
    }
}

impl Default for ProtocolSession {
    fn default() -> Self {
        Self::new()
    }
}