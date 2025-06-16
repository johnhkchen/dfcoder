use crate::*;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Transport layer abstraction for MCP
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Send a message
    async fn send(&self, message: &str) -> Result<(), McpError>;
    
    /// Receive a message
    async fn receive(&self) -> Result<String, McpError>;
    
    /// Close the transport
    async fn close(&self) -> Result<(), McpError>;
    
    /// Check if the transport is connected
    fn is_connected(&self) -> bool;
}

/// Stdio transport implementation
#[derive(Debug)]
pub struct StdioTransport {
    stdin_sender: Arc<Mutex<mpsc::UnboundedSender<String>>>,
    stdout_receiver: Arc<Mutex<mpsc::UnboundedReceiver<String>>>,
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        let (stdin_sender, stdin_receiver) = mpsc::unbounded_channel();
        let (stdout_sender, stdout_receiver) = mpsc::unbounded_channel();
        
        // Start background tasks for stdio handling
        tokio::spawn(Self::handle_stdin(stdin_receiver));
        tokio::spawn(Self::handle_stdout(stdout_sender));
        
        Self {
            stdin_sender: Arc::new(Mutex::new(stdin_sender)),
            stdout_receiver: Arc::new(Mutex::new(stdout_receiver)),
            connected: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }
    
    async fn handle_stdin(mut receiver: mpsc::UnboundedReceiver<String>) {
        use tokio::io::{AsyncWriteExt, stdout};
        
        let mut stdout = stdout();
        while let Some(message) = receiver.recv().await {
            if let Err(e) = stdout.write_all(message.as_bytes()).await {
                tracing::error!("Failed to write to stdout: {}", e);
                break;
            }
            if let Err(e) = stdout.write_all(b"\n").await {
                tracing::error!("Failed to write newline to stdout: {}", e);
                break;
            }
            if let Err(e) = stdout.flush().await {
                tracing::error!("Failed to flush stdout: {}", e);
                break;
            }
        }
    }
    
    async fn handle_stdout(sender: mpsc::UnboundedSender<String>) {
        use tokio::io::{AsyncBufReadExt, BufReader, stdin};
        
        let stdin = stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    // Remove the trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    
                    if sender.send(line.clone()).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Transport for StdioTransport {
    async fn send(&self, message: &str) -> Result<(), McpError> {
        let sender = self.stdin_sender.lock().await;
        sender.send(message.to_string())
            .map_err(|e| McpError::TransportError(format!("Failed to send message: {}", e)))?;
        Ok(())
    }
    
    async fn receive(&self) -> Result<String, McpError> {
        let mut receiver = self.stdout_receiver.lock().await;
        receiver.recv().await
            .ok_or_else(|| McpError::TransportError("Channel closed".to_string()))
    }
    
    async fn close(&self) -> Result<(), McpError> {
        self.connected.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket transport implementation
#[derive(Debug)]
pub struct WebSocketTransport {
    url: String,
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl WebSocketTransport {
    pub fn new(url: String) -> Self {
        Self {
            url,
            connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub async fn connect(&self) -> Result<(), McpError> {
        // WebSocket connection logic would go here
        tracing::info!("Connecting to WebSocket: {}", self.url);
        self.connected.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Transport for WebSocketTransport {
    async fn send(&self, message: &str) -> Result<(), McpError> {
        if !self.is_connected() {
            return Err(McpError::TransportError("WebSocket not connected".to_string()));
        }
        
        // WebSocket send logic would go here
        tracing::debug!("Sending WebSocket message: {}", message);
        Ok(())
    }
    
    async fn receive(&self) -> Result<String, McpError> {
        if !self.is_connected() {
            return Err(McpError::TransportError("WebSocket not connected".to_string()));
        }
        
        // WebSocket receive logic would go here
        // For now, return a placeholder
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok("{}".to_string())
    }
    
    async fn close(&self) -> Result<(), McpError> {
        self.connected.store(false, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("WebSocket connection closed");
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// TCP transport implementation
#[derive(Debug)]
pub struct TcpTransport {
    address: String,
    port: u16,
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl TcpTransport {
    pub fn new(address: String, port: u16) -> Self {
        Self {
            address,
            port,
            connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub async fn connect(&self) -> Result<(), McpError> {
        // TCP connection logic would go here
        tracing::info!("Connecting to TCP: {}:{}", self.address, self.port);
        self.connected.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Transport for TcpTransport {
    async fn send(&self, message: &str) -> Result<(), McpError> {
        if !self.is_connected() {
            return Err(McpError::TransportError("TCP not connected".to_string()));
        }
        
        // TCP send logic would go here
        tracing::debug!("Sending TCP message: {}", message);
        Ok(())
    }
    
    async fn receive(&self) -> Result<String, McpError> {
        if !self.is_connected() {
            return Err(McpError::TransportError("TCP not connected".to_string()));
        }
        
        // TCP receive logic would go here
        // For now, return a placeholder
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok("{}".to_string())
    }
    
    async fn close(&self) -> Result<(), McpError> {
        self.connected.store(false, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("TCP connection closed");
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Transport factory for creating transports based on configuration
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport from configuration
    pub fn create_transport(config: &TransportConfig) -> Result<Box<dyn Transport>, McpError> {
        match config.transport_type {
            TransportType::Stdio => Ok(Box::new(StdioTransport::new())),
            TransportType::WebSocket => {
                let url = config.address.as_ref()
                    .ok_or_else(|| McpError::TransportError("WebSocket address required".to_string()))?;
                let port = config.port.unwrap_or(80);
                let full_url = if url.contains("://") {
                    url.clone()
                } else {
                    format!("ws://{}:{}", url, port)
                };
                Ok(Box::new(WebSocketTransport::new(full_url)))
            }
            TransportType::Tcp => {
                let address = config.address.as_ref()
                    .ok_or_else(|| McpError::TransportError("TCP address required".to_string()))?;
                let port = config.port
                    .ok_or_else(|| McpError::TransportError("TCP port required".to_string()))?;
                Ok(Box::new(TcpTransport::new(address.clone(), port)))
            }
            TransportType::Unix => {
                // Unix socket implementation would go here
                Err(McpError::TransportError("Unix socket transport not implemented".to_string()))
            }
        }
    }
}

/// Transport message framing for protocols that need it
pub struct MessageFramer {
    buffer: String,
    max_message_size: usize,
}

impl MessageFramer {
    pub fn new(max_message_size: usize) -> Self {
        Self {
            buffer: String::new(),
            max_message_size,
        }
    }
    
    /// Add data to the buffer and extract complete messages
    pub fn add_data(&mut self, data: &str) -> Result<Vec<String>, McpError> {
        self.buffer.push_str(data);
        
        if self.buffer.len() > self.max_message_size {
            return Err(McpError::TransportError("Message too large".to_string()));
        }
        
        let mut messages = Vec::new();
        
        // For JSON-RPC, messages are typically line-delimited
        while let Some(newline_pos) = self.buffer.find('\n') {
            let message = self.buffer[..newline_pos].trim().to_string();
            self.buffer.drain(..=newline_pos);
            
            if !message.is_empty() {
                messages.push(message);
            }
        }
        
        Ok(messages)
    }
    
    /// Frame a message for transmission
    pub fn frame_message(&self, message: &str) -> String {
        format!("{}\n", message)
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Transport connection manager
pub struct ConnectionManager {
    transport: Box<dyn Transport>,
    framer: MessageFramer,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl ConnectionManager {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            framer: MessageFramer::new(1024 * 1024), // 1MB max message size
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    /// Start the connection manager
    pub async fn start(&mut self) -> Result<(), McpError> {
        self.is_running.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    /// Stop the connection manager
    pub async fn stop(&mut self) -> Result<(), McpError> {
        self.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
        self.transport.close().await
    }
    
    /// Send a message
    pub async fn send_message(&mut self, message: &str) -> Result<(), McpError> {
        let framed_message = self.framer.frame_message(message);
        self.transport.send(&framed_message).await
    }
    
    /// Receive messages
    pub async fn receive_messages(&mut self) -> Result<Vec<String>, McpError> {
        let data = self.transport.receive().await?;
        self.framer.add_data(&data)
    }
    
    /// Check if the connection is active
    pub fn is_active(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::Relaxed) && 
        self.transport.is_connected()
    }
}