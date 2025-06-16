use std::time::Instant;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Mock pane for testing pane monitoring scenarios
#[derive(Debug, Clone)]
pub struct MockPane {
    pub id: u32,
    pub content: String,
    pub last_update: Instant,
    pub is_active: bool,
    pub has_errors: bool,
    pub command_history: VecDeque<String>,
    pub output_history: VecDeque<PaneOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PaneOutput {
    pub content: String,
    #[serde(skip)]
    pub timestamp: Instant,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Stdout,
    Stderr,
    Command,
    System,
}

impl MockPane {
    /// Create a new mock pane
    pub fn new(id: u32) -> Self {
        Self {
            id,
            content: String::new(),
            last_update: Instant::now(),
            is_active: false,
            has_errors: false,
            command_history: VecDeque::new(),
            output_history: VecDeque::new(),
        }
    }
    
    /// Set the pane content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.last_update = Instant::now();
        self
    }
    
    /// Mark the pane as active
    pub fn active(mut self) -> Self {
        self.is_active = true;
        self
    }
    
    /// Mark the pane as having errors
    pub fn with_errors(mut self) -> Self {
        self.has_errors = true;
        self
    }
    
    /// Add a command to the history
    pub fn execute_command(&mut self, command: impl Into<String>) {
        let cmd = command.into();
        self.command_history.push_back(cmd.clone());
        self.output_history.push_back(PaneOutput {
            content: format!("$ {}", cmd),
            timestamp: Instant::now(),
            output_type: OutputType::Command,
        });
        self.last_update = Instant::now();
    }
    
    /// Add output to the pane
    pub fn add_output(&mut self, content: impl Into<String>, output_type: OutputType) {
        let content = content.into();
        self.content.push_str(&content);
        self.content.push('\n');
        
        self.output_history.push_back(PaneOutput {
            content: content.clone(),
            timestamp: Instant::now(),
            output_type: output_type.clone(),
        });
        
        // Check for errors in stderr
        if matches!(output_type, OutputType::Stderr) {
            self.has_errors = true;
        }
        
        self.last_update = Instant::now();
    }
    
    /// Add stdout output
    pub fn add_stdout(&mut self, content: impl Into<String>) {
        self.add_output(content, OutputType::Stdout);
    }
    
    /// Add stderr output
    pub fn add_stderr(&mut self, content: impl Into<String>) {
        self.add_output(content, OutputType::Stderr);
    }
    
    /// Add system message
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.add_output(content, OutputType::System);
    }
    
    /// Simulate Claude Code agent output
    pub fn simulate_claude_output(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.add_stdout(format!("Claude: {}", message));
    }
    
    /// Simulate typing by an agent
    pub fn simulate_typing(&mut self, text: impl Into<String>) {
        let text = text.into();
        // Simulate gradual typing
        for chunk in text.split_whitespace() {
            self.content.push_str(chunk);
            self.content.push(' ');
            self.last_update = Instant::now();
        }
    }
    
    /// Clear the pane content
    pub fn clear(&mut self) {
        self.content.clear();
        self.has_errors = false;
        self.last_update = Instant::now();
    }
    
    /// Check if pane contains specific text
    pub fn contains(&self, text: &str) -> bool {
        self.content.contains(text)
    }
    
    /// Get the most recent output of a specific type
    pub fn last_output_of_type(&self, output_type: OutputType) -> Option<&PaneOutput> {
        self.output_history.iter().rev().find(|output| {
            std::mem::discriminant(&output.output_type) == std::mem::discriminant(&output_type)
        })
    }
    
    /// Get all commands executed in this pane
    pub fn get_commands(&self) -> Vec<&String> {
        self.command_history.iter().collect()
    }
    
    /// Check if the pane has been idle for a duration
    pub fn is_idle_for(&self, duration: std::time::Duration) -> bool {
        self.last_update.elapsed() >= duration
    }
}

/// Builder for pane test scenarios
pub struct PaneScenarioBuilder {
    pane: MockPane,
}

impl PaneScenarioBuilder {
    pub fn new(id: u32) -> Self {
        Self {
            pane: MockPane::new(id),
        }
    }
    
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.pane = self.pane.with_content(content);
        self
    }
    
    pub fn that_is_active(mut self) -> Self {
        self.pane = self.pane.active();
        self
    }
    
    pub fn with_errors(mut self) -> Self {
        self.pane = self.pane.with_errors();
        self
    }
    
    pub fn with_command_history(mut self, commands: Vec<&str>) -> Self {
        for cmd in commands {
            self.pane.execute_command(cmd);
        }
        self
    }
    
    pub fn build(self) -> MockPane {
        self.pane
    }
}

/// Convenient factory methods for common pane scenarios
impl MockPane {
    /// Create a pane with a Claude Code session
    pub fn claude_session(id: u32) -> Self {
        let mut pane = Self::new(id).active();
        pane.simulate_claude_output("I'm ready to help with your coding tasks!");
        pane
    }
    
    /// Create a pane with build errors
    pub fn with_build_errors(id: u32) -> Self {
        let mut pane = Self::new(id).with_errors();
        pane.execute_command("cargo build");
        pane.add_stderr("error[E0308]: mismatched types");
        pane.add_stderr("expected `i32`, found `&str`");
        pane
    }
    
    /// Create a pane with test failures
    pub fn with_test_failures(id: u32) -> Self {
        let mut pane = Self::new(id).with_errors();
        pane.execute_command("cargo test");
        pane.add_stderr("test result: FAILED. 2 passed; 3 failed; 0 ignored");
        pane
    }
    
    /// Create a pane that's been idle
    pub fn idle_pane(id: u32) -> Self {
        let mut pane = Self::new(id);
        pane.last_update = Instant::now() - std::time::Duration::from_secs(300); // 5 minutes
        pane
    }
}

impl Default for PaneOutput {
    fn default() -> Self {
        Self {
            content: String::new(),
            timestamp: Instant::now(),
            output_type: OutputType::Stdout,
        }
    }
}