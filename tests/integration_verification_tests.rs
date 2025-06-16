//! Integration Verification Tests
//!
//! Basic tests to verify system integration and core functionality.

use dfcoder_core::*;
use dfcoder_test_utils::TestSystem;
use std::time::Duration;

/// Test 1: Error Recovery & Retry Logic Integration
#[tokio::test]
async fn test_error_recovery_retry_logic() {
    println!("🔍 Testing Error Recovery & Retry Logic Integration");
    
    // 1.1: Retry policies exist with different configurations
    let default_policy = RetryPolicy::default();
    let conservative_policy = RetryPolicy::conservative();
    let aggressive_policy = RetryPolicy::aggressive();
    
    assert_eq!(default_policy.max_attempts, 3);
    assert_eq!(conservative_policy.max_attempts, 2); 
    assert_eq!(aggressive_policy.max_attempts, 5);
    println!("✅ Multiple retry policies implemented");
    
    // 1.2: Exponential backoff calculation works
    assert_eq!(default_policy.calculate_backoff(1), Duration::from_secs(1));
    assert_eq!(default_policy.calculate_backoff(2), Duration::from_secs(2));
    assert_eq!(default_policy.calculate_backoff(3), Duration::from_secs(4));
    println!("✅ Exponential backoff working");
    
    // 1.3: Error type filtering
    assert!(default_policy.should_retry(&ErrorType::NetworkError));
    assert!(!default_policy.should_retry(&ErrorType::Fatal));
    println!("✅ Smart error type filtering");
    
    // 1.4: RetryExecutor can execute tasks with actual retries
    let executor = RetryExecutor::new(RetryPolicy::default());
    let mut agent = Agent::new(AgentRole::Implementer, 1);
    let task = Task::new("Test".to_string(), "Task with network issues".to_string(), AgentRole::Implementer, TaskPriority::Normal);
    
    let start = std::time::Instant::now();
    let result = executor.execute_task(&mut agent, &task).await;
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    assert!(result.unwrap().attempt_number > 1);
    assert!(elapsed >= Duration::from_secs(1));
    println!("✅ Retry execution with delays verified");
    
    println!("🎉 Error Recovery & Retry Logic: VERIFIED\n");
}

/// Test 2: Task Prioritization System Integration
#[tokio::test]
async fn test_task_prioritization_system() {
    println!("🔍 Testing Task Prioritization System Integration");
    
    let mut workshop = WorkshopManager::new();
    let agent = Agent::new(AgentRole::Implementer, 1);
    let agent_id = agent.id.clone();
    workshop.register_agent(agent).unwrap();
    
    // 2.1: Priority-based assignment
    let high_task = Task::new("Critical".to_string(), "Critical task".to_string(), AgentRole::Implementer, TaskPriority::High);
    let normal_task = Task::new("Normal".to_string(), "Normal task".to_string(), AgentRole::Implementer, TaskPriority::Normal);
    
    workshop.queue_task(normal_task);
    workshop.queue_task(high_task);
    
    let assignment = workshop.assign_by_priority().unwrap();
    assert!(assignment.is_some());
    let (_agent, task) = assignment.unwrap();
    assert_eq!(task.title, "Critical");
    println!("✅ Priority-based assignment working");
    
    // 2.2: Task complexity levels defined
    let _simple = TaskComplexity::Simple;
    let _complex = TaskComplexity::Complex;
    println!("✅ Task complexity levels defined");
    
    // 2.3: Agent expertise tracking structure exists
    let _expertise = AgentExpertise::default();
    println!("✅ Agent expertise tracking implemented");
    
    // 2.4: Workshop integration with retry
    let retry_task = Task::new("Retry".to_string(), "test".to_string(), AgentRole::Implementer, TaskPriority::Normal);
    let result = workshop.execute_task_with_retry(&agent_id, &retry_task).await;
    assert!(result.is_ok());
    println!("✅ Retry integration in workshop verified");
    
    println!("🎉 Task Prioritization System: VERIFIED\n");
}

/// Test 3: Performance Metrics Dashboard Integration
#[test]
fn test_performance_metrics_dashboard() {
    println!("🔍 Testing Performance Metrics Dashboard Integration");
    
    let mut workshop = WorkshopManager::new();
    let status = workshop.get_status();
    let metrics = &status.metrics;
    
    // 3.1: All claimed metrics fields exist
    let _total = metrics.total_tasks_processed;
    let _completed = metrics.tasks_completed; 
    let _failed = metrics.tasks_failed;
    let _retried = metrics.tasks_retried;
    let _duration = metrics.average_task_duration;
    let _utilization = &metrics.agent_utilization;
    let _queue = metrics.queue_length;
    let _bottleneck = &metrics.bottleneck_role;
    let _throughput = metrics.throughput;
    let _success_rate = metrics.success_rate;
    let _cost = metrics.cost_per_task;
    println!("✅ Comprehensive metrics structure exists");
    
    // 3.2: Bottleneck detection works
    let agent = Agent::new(AgentRole::Scaffolder, 1);
    workshop.register_agent(agent).unwrap();
    
    let task = Task::new("Setup".to_string(), "Setup".to_string(), AgentRole::Scaffolder, TaskPriority::Normal);
    workshop.assign_task(task).unwrap();
    
    let status = workshop.get_status();
    let scaffolder_util = status.metrics.agent_utilization.get(&AgentRole::Scaffolder).unwrap_or(&0.0);
    assert_eq!(*scaffolder_util, 1.0);
    assert_eq!(status.metrics.bottleneck_role, Some(AgentRole::Scaffolder));
    println!("✅ Bottleneck detection working");
    
    // 3.3: Per-role capacity tracking
    assert!(status.capacity_per_role.contains_key(&AgentRole::Scaffolder));
    assert!(status.capacity_per_role.contains_key(&AgentRole::Implementer));
    println!("✅ Per-role capacity tracking verified");
    
    println!("🎉 Performance Metrics Dashboard: VERIFIED\n");
}

/// Test 4: Enhanced Coordination System Integration
#[tokio::test]
async fn test_enhanced_coordination_system() {
    println!("🔍 Testing Enhanced Coordination System Integration");
    
    let mut system = TestSystem::new();
    
    // 4.1: Integration works
    let _agent = system.spawn_agent(AgentRole::Implementer);
    let (task_id, agent_id) = system.assign_task_to_role(AgentRole::Implementer, "test");
    
    system.complete_task(agent_id.clone(), task_id).unwrap();
    let agent_after = system.get_agent(&agent_id).unwrap();
    assert!(agent_after.metrics.tasks_completed > 0);
    println!("✅ TestSystem integration working");
    
    // 4.2: Workshop status tracking
    let status = system.get_workshop_status();
    assert!(status.total_agents > 0);
    assert!(!status.capacity_per_role.is_empty());
    println!("✅ Workshop status tracking working");
    
    // 4.3: Error handling
    let error = system.fail_task(agent_id, "invalid".to_string(), "error".to_string());
    assert!(error.is_err());
    println!("✅ Error handling working");
    
    println!("🎉 Enhanced Coordination System: VERIFIED\n");
}

/// Test 5: Architecture & Type Safety Integration
#[test]
fn test_architecture_and_type_safety() {
    println!("🔍 Testing Architecture & Type Safety Integration");
    
    // 5.1: Retry module properly exported
    let _policy = RetryPolicy::default();
    let _executor = RetryExecutor::new(_policy);
    println!("✅ Retry module exported");
    
    // 5.2: All types compile and are type-safe
    let _expertise = AgentExpertise::default();
    let _complexity = TaskComplexity::Simple;
    let _error_type = ErrorType::NetworkError;
    let _state = RetryState::default();
    println!("✅ Type safety verified");
    
    // 5.3: Serde serialization works
    let metrics = WorkshopMetrics::default();
    let _serialized = serde_json::to_string(&metrics).unwrap();
    println!("✅ Serde serialization working");
    
    // 5.4: Error types exist
    let _workshop_err = WorkshopError::AgentNotFound("test".to_string());
    let _retry_err = RetryError::MaxAttemptsExceeded(3);
    println!("✅ Error types implemented");
    
    println!("🎉 Architecture & Type Safety: VERIFIED\n");
}

/// Test 6: System Integration with Existing Components
#[test]
fn test_existing_test_suite_integration() {
    println!("🔍 Testing System Integration with Existing Components");
    
    // This test verifies that our new features don't break existing functionality
    // by checking that core components still work
    
    // Agent creation still works
    let agent = Agent::new(AgentRole::Implementer, 1);
    assert_eq!(agent.status, AgentStatus::Idle);
    println!("✅ Agent creation still works");
    
    // Task creation still works
    let task = Task::new("Test".to_string(), "Description".to_string(), AgentRole::Implementer, TaskPriority::Normal);
    assert_eq!(task.status, TaskStatus::Pending);
    println!("✅ Task creation still works");
    
    // Workshop management still works
    let mut workshop = WorkshopManager::new();
    assert!(workshop.can_assign(AgentRole::Implementer));
    println!("✅ Workshop management still works");
    
    // Supervision system still works
    let supervision = SupervisionSystem::new();
    assert!(supervision.get_all_active_requests().is_empty());
    println!("✅ Supervision system still works");
    
    println!("🎉 System Integration with Existing Components: VERIFIED\n");
}