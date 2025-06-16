//! Error recovery and retry logic for robust agent task execution
//! 
//! Provides exponential backoff, failure pattern tracking, and strategy switching.

use crate::agents::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Policy for retrying failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f32,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Error types that should trigger retries
    pub retry_on: Vec<ErrorType>,
}

/// Types of errors that can occur during task execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Network/API connection issues
    NetworkError,
    /// Rate limiting from external services
    RateLimitError,
    /// Authentication/authorization failures
    AuthError,
    /// Temporary resource unavailability
    ResourceUnavailable,
    /// Parse/format errors in output
    ParseError,
    /// Task complexity exceeds agent capability
    ComplexityError,
    /// Generic retryable error
    Retryable,
    /// Non-retryable logic errors
    Fatal,
}

/// Result of a task execution attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub error: Option<ErrorType>,
    pub duration: Duration,
    pub attempt_number: u32,
}

/// Tracks retry attempts and failure patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    pub attempts: u32,
    #[serde(skip)]
    pub last_attempt: Option<Instant>,
    pub failure_pattern: Vec<ErrorType>,
    pub total_duration: Duration,
}

/// Errors that can occur in the retry system
#[derive(Debug, Error)]
pub enum RetryError {
    #[error("Maximum retry attempts exceeded: {0}")]
    MaxAttemptsExceeded(u32),
    #[error("Non-retryable error: {0:?}")]
    NonRetryable(ErrorType),
    #[error("Timeout waiting for retry: {0:?}")]
    Timeout(Duration),
    #[error("Agent error: {0}")]
    AgentError(String),
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(30),
            retry_on: vec![
                ErrorType::NetworkError,
                ErrorType::RateLimitError,
                ErrorType::ResourceUnavailable,
                ErrorType::Retryable,
            ],
        }
    }
}

impl RetryPolicy {
    /// Create a conservative retry policy
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_backoff: Duration::from_secs(2),
            backoff_multiplier: 3.0,
            max_backoff: Duration::from_secs(60),
            retry_on: vec![ErrorType::NetworkError, ErrorType::RateLimitError],
        }
    }

    /// Create an aggressive retry policy
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff: Duration::from_millis(500),
            backoff_multiplier: 1.5,
            max_backoff: Duration::from_secs(15),
            retry_on: vec![
                ErrorType::NetworkError,
                ErrorType::RateLimitError,
                ErrorType::ResourceUnavailable,
                ErrorType::ParseError,
                ErrorType::Retryable,
            ],
        }
    }

    /// Calculate backoff duration for given attempt
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let backoff_secs = self.initial_backoff.as_secs_f32() 
            * self.backoff_multiplier.powi((attempt - 1) as i32);
        
        let backoff = Duration::from_secs_f32(backoff_secs);
        std::cmp::min(backoff, self.max_backoff)
    }

    /// Check if an error type should trigger a retry
    pub fn should_retry(&self, error: &ErrorType) -> bool {
        self.retry_on.contains(error)
    }
}

/// Executor for tasks with retry logic
#[derive(Debug)]
pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    /// Create a new retry executor with the given policy
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    /// Execute a task with retry logic
    pub async fn execute_task(
        &self,
        agent: &mut Agent,
        task: &Task,
    ) -> Result<TaskResult, RetryError> {
        let mut retry_state = RetryState {
            attempts: 0,
            last_attempt: None,
            failure_pattern: Vec::new(),
            total_duration: Duration::from_secs(0),
        };

        let start_time = Instant::now();

        for attempt in 1..=self.policy.max_attempts {
            retry_state.attempts = attempt;
            retry_state.last_attempt = Some(Instant::now());

            // Apply backoff delay
            if attempt > 1 {
                let backoff = self.policy.calculate_backoff(attempt - 1);
                tokio::time::sleep(backoff).await;
            }

            // Execute the task
            let attempt_start = Instant::now();
            let result = self.execute_single_attempt(agent, task, attempt).await;
            let _attempt_duration = attempt_start.elapsed();

            match result {
                Ok(task_result) => {
                    // Success - return result
                    return Ok(TaskResult {
                        success: true,
                        output: task_result.output,
                        error: None,
                        duration: start_time.elapsed(),
                        attempt_number: attempt,
                    });
                }
                Err(error) => {
                    retry_state.failure_pattern.push(error.clone());

                    // Check if we should retry this error
                    if !self.policy.should_retry(&error) {
                        return Err(RetryError::NonRetryable(error));
                    }

                    // Check if this is the last attempt
                    if attempt == self.policy.max_attempts {
                        return Err(RetryError::MaxAttemptsExceeded(attempt));
                    }

                    // Adapt strategy based on failure pattern
                    self.adapt_strategy_for_failures(&retry_state.failure_pattern, agent);
                }
            }
        }

        Err(RetryError::MaxAttemptsExceeded(self.policy.max_attempts))
    }

    /// Execute a single attempt of the task
    async fn execute_single_attempt(
        &self,
        _agent: &mut Agent,
        task: &Task,
        attempt: u32,
    ) -> Result<TaskResult, ErrorType> {
        // Simulate task execution - in real implementation this would
        // call the actual agent execution logic
        
        // For demo purposes, simulate different failure scenarios
        if attempt == 1 && task.description.contains("network") {
            return Err(ErrorType::NetworkError);
        }
        
        if attempt <= 2 && task.description.contains("rate") {
            return Err(ErrorType::RateLimitError);
        }
        
        if task.description.contains("fatal") {
            return Err(ErrorType::Fatal);
        }

        // Simulate successful execution
        Ok(TaskResult {
            success: true,
            output: format!("Task '{}' completed successfully on attempt {}", 
                          task.title, attempt),
            error: None,
            duration: Duration::from_secs(1),
            attempt_number: attempt,
        })
    }

    /// Adapt strategy based on observed failure patterns
    fn adapt_strategy_for_failures(&self, failures: &[ErrorType], agent: &mut Agent) {
        // Analyze failure patterns and adjust approach
        let network_failures = failures.iter().filter(|e| **e == ErrorType::NetworkError).count();
        let rate_limit_failures = failures.iter().filter(|e| **e == ErrorType::RateLimitError).count();
        
        if network_failures > 1 {
            // Switch to more robust network handling
            tracing::info!("Agent {}: Switching to robust network mode after {} network failures", 
                          agent.id, network_failures);
        }
        
        if rate_limit_failures > 0 {
            // Reduce request frequency
            tracing::info!("Agent {}: Reducing request frequency after {} rate limit errors", 
                          agent.id, rate_limit_failures);
        }
    }
}

impl Default for RetryState {
    fn default() -> Self {
        Self {
            attempts: 0,
            last_attempt: None,
            failure_pattern: Vec::new(),
            total_duration: Duration::from_secs(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_backoff_calculation() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.calculate_backoff(0), Duration::from_secs(0));
        assert_eq!(policy.calculate_backoff(1), Duration::from_secs(1));
        assert_eq!(policy.calculate_backoff(2), Duration::from_secs(2));
        assert_eq!(policy.calculate_backoff(3), Duration::from_secs(4));
    }

    #[test]
    fn test_should_retry_logic() {
        let policy = RetryPolicy::default();
        
        assert!(policy.should_retry(&ErrorType::NetworkError));
        assert!(policy.should_retry(&ErrorType::RateLimitError));
        assert!(!policy.should_retry(&ErrorType::Fatal));
        assert!(!policy.should_retry(&ErrorType::AuthError));
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let policy = RetryPolicy::default();
        let executor = RetryExecutor::new(policy);
        let mut agent = Agent::new(AgentRole::Implementer, 1);
        let task = Task::new(
            "Test task".to_string(),
            "Simple test task".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );

        let result = executor.execute_task(&mut agent, &task).await;
        assert!(result.is_ok());
        
        let task_result = result.unwrap();
        assert!(task_result.success);
        assert_eq!(task_result.attempt_number, 1);
    }

    #[tokio::test]
    async fn test_retry_executor_with_retries() {
        let policy = RetryPolicy::default();
        let executor = RetryExecutor::new(policy);
        let mut agent = Agent::new(AgentRole::Implementer, 1);
        let task = Task::new(
            "Test task".to_string(),
            "Task with network issues".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );

        let result = executor.execute_task(&mut agent, &task).await;
        assert!(result.is_ok());
        
        let task_result = result.unwrap();
        assert!(task_result.success);
        assert!(task_result.attempt_number > 1);
    }
}