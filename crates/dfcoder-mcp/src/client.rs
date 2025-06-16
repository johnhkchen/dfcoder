use crate::*;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::process::Stdio;

/// MCP client implementation
#[derive(Debug)]
pub struct McpClient {
    protocol: McpProtocol,
    session: ProtocolSession,
    connection: ClientConnection,
}

/// Client connection types
#[derive(Debug)]
pub enum ClientConnection {
    Stdio {
        process: tokio::process::Child,
        stdin: tokio::process::ChildStdin,
        stdout: BufReader<tokio::process::ChildStdout>,
    },
    WebSocket {
        url: String,
    },
    Tcp {
        address: String,
        port: u16,
    },
}

impl McpClient {
    /// Create a new MCP client connected to a server
    pub async fn new(server_uri: &str) -> Result<Self, McpError> {
        let protocol = McpProtocol::new("2024-11-05".to_string());
        let session = ProtocolSession::new();
        
        let connection = if server_uri.starts_with("stdio://") {
            Self::create_stdio_connection(server_uri).await?
        } else if server_uri.starts_with("ws://") || server_uri.starts_with("wss://") {
            Self::create_websocket_connection(server_uri).await?
        } else if server_uri.starts_with("tcp://") {
            Self::create_tcp_connection(server_uri).await?
        } else {
            return Err(McpError::TransportError(format!("Unsupported URI scheme: {}", server_uri)));
        };
        
        let mut client = Self {
            protocol,
            session,
            connection,
        };
        
        // Initialize the connection
        client.initialize().await?;
        
        Ok(client)
    }
    
    /// Initialize the MCP connection
    async fn initialize(&mut self) -> Result<(), McpError> {
        let capabilities = ClientCapabilities::default();
        let init_request = self.protocol.create_initialize_request(capabilities);
        
        let response = self.send_request(init_request).await?;
        
        if let Some(result) = response.result {
            if let Some(server_caps) = result.get("capabilities") {
                let capabilities: ServerCapabilities = serde_json::from_value(server_caps.clone())?;
                self.session.set_capabilities(capabilities);
            }
            self.session.set_state(ProtocolState::Initialized);
            tracing::info!("MCP client initialized successfully");
        } else if let Some(error) = response.error {
            return Err(McpError::ProtocolError(format!("Initialization failed: {}", error.message)));
        }
        
        Ok(())
    }
    
    /// List available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        self.ensure_initialized()?;
        
        let request = self.protocol.create_list_resources_request();
        let response = self.send_request(request).await?;
        
        if let Some(result) = response.result {
            if let Some(resources) = result.get("resources") {
                let resource_list: Vec<Resource> = serde_json::from_value(resources.clone())?;
                return Ok(resource_list);
            }
        }
        
        Ok(Vec::new())
    }
    
    /// Read a specific resource
    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContent, McpError> {
        self.ensure_initialized()?;
        
        let request = self.protocol.create_read_resource_request(uri);
        let response = self.send_request(request).await?;
        
        if let Some(result) = response.result {
            if let Some(contents) = result.get("contents") {
                if let Some(content_array) = contents.as_array() {
                    if let Some(first_content) = content_array.first() {
                        let content: ResourceContent = serde_json::from_value(first_content.clone())?;
                        return Ok(content);
                    }
                }
            }
        } else if let Some(error) = response.error {
            return Err(McpError::ResourceError(error.message));
        }
        
        Err(McpError::ResourceError("No content returned".to_string()))
    }
    
    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult, McpError> {
        self.ensure_initialized()?;
        
        let request = self.protocol.create_tool_call_request(name, arguments);
        let response = self.send_request(request).await?;
        
        if let Some(result) = response.result {
            let tool_result: ToolResult = serde_json::from_value(result)?;
            return Ok(tool_result);
        } else if let Some(error) = response.error {
            return Err(McpError::ToolError(error.message));
        }
        
        Err(McpError::ToolError("No result returned".to_string()))
    }
    
    /// Get a prompt
    pub async fn get_prompt(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<PromptResult, McpError> {
        self.ensure_initialized()?;
        
        let request = self.protocol.create_get_prompt_request(name, arguments);
        let response = self.send_request(request).await?;
        
        if let Some(result) = response.result {
            let prompt_result: PromptResult = serde_json::from_value(result)?;
            return Ok(prompt_result);
        } else if let Some(error) = response.error {
            return Err(McpError::ResourceError(error.message));
        }
        
        Err(McpError::ResourceError("No prompt returned".to_string()))
    }
    
    /// Send a request and wait for response
    async fn send_request(&self, request: McpMessage) -> Result<McpMessage, McpError> {
        match &self.connection {
            ClientConnection::Stdio { stdin, stdout, .. } => {
                // This is a simplified implementation
                // In a real implementation, we'd need proper async handling
                let request_str = self.protocol.serialize_message(&request)?;
                // stdin.write_all(request_str.as_bytes()).await?;
                // stdin.write_all(b"\n").await?;
                
                // For now, return a mock response
                Ok(McpMessage {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    method: None,
                    params: None,
                    result: Some(serde_json::json!({})),
                    error: None,
                })
            }
            ClientConnection::WebSocket { .. } => {
                // WebSocket implementation would go here
                Err(McpError::TransportError("WebSocket not implemented".to_string()))
            }
            ClientConnection::Tcp { .. } => {
                // TCP implementation would go here
                Err(McpError::TransportError("TCP not implemented".to_string()))
            }
        }
    }
    
    async fn create_stdio_connection(uri: &str) -> Result<ClientConnection, McpError> {
        // Parse the command from the URI
        let command_part = uri.strip_prefix("stdio://").unwrap_or(uri);
        let parts: Vec<&str> = command_part.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(McpError::TransportError("Empty command".to_string()));
        }
        
        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        
        let mut process = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| McpError::TransportError(format!("Failed to spawn process: {}", e)))?;
        
        let stdin = process.stdin.take()
            .ok_or_else(|| McpError::TransportError("Failed to get stdin".to_string()))?;
        
        let stdout = process.stdout.take()
            .ok_or_else(|| McpError::TransportError("Failed to get stdout".to_string()))?;
        
        let stdout = BufReader::new(stdout);
        
        Ok(ClientConnection::Stdio {
            process,
            stdin,
            stdout,
        })
    }
    
    async fn create_websocket_connection(uri: &str) -> Result<ClientConnection, McpError> {
        Ok(ClientConnection::WebSocket {
            url: uri.to_string(),
        })
    }
    
    async fn create_tcp_connection(uri: &str) -> Result<ClientConnection, McpError> {
        let url = url::Url::parse(uri)
            .map_err(|e| McpError::TransportError(format!("Invalid TCP URI: {}", e)))?;
        
        let host = url.host_str()
            .ok_or_else(|| McpError::TransportError("Missing host in TCP URI".to_string()))?;
        
        let port = url.port()
            .ok_or_else(|| McpError::TransportError("Missing port in TCP URI".to_string()))?;
        
        Ok(ClientConnection::Tcp {
            address: host.to_string(),
            port,
        })
    }
    
    fn ensure_initialized(&self) -> Result<(), McpError> {
        if !self.session.is_initialized() {
            return Err(McpError::ProtocolError("Client not initialized".to_string()));
        }
        Ok(())
    }
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: Option<String>,
    pub blob: Option<String>,
}

/// Resource representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

/// Client builder for easier configuration
pub struct McpClientBuilder {
    server_uri: String,
    timeout: Option<std::time::Duration>,
    retry_attempts: u32,
}

impl McpClientBuilder {
    pub fn new(server_uri: impl Into<String>) -> Self {
        Self {
            server_uri: server_uri.into(),
            timeout: None,
            retry_attempts: 3,
        }
    }
    
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }
    
    pub async fn connect(self) -> Result<McpClient, McpError> {
        let mut last_error = None;
        
        for attempt in 0..self.retry_attempts {
            match McpClient::new(&self.server_uri).await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.retry_attempts - 1 {
                        tracing::warn!("Connection attempt {} failed, retrying...", attempt + 1);
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| McpError::TransportError("Connection failed".to_string())))
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let ClientConnection::Stdio { process, .. } = &mut self.connection {
            let _ = process.kill();
        }
    }
}