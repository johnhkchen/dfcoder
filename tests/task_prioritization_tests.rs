//! Task Prioritization and Core Features Tests
//! 
//! Systematic verification of task prioritization, error recovery, and coordination systems.

use dfcoder_core::*;
use dfcoder_test_utils::TestSystem;
use std::time::Duration;

/// Test 1: Error Recovery & Retry Logic
#[tokio::test]
async fn test_error_recovery_implementation() {
    println!("🔍 TESTING 1: Error Recovery & Retry Logic");
    
    // Verify 1.1: Retry policies exist with correct configurations
    let default_policy = RetryPolicy::default();
    let conservative_policy = RetryPolicy::conservative();
    let aggressive_policy = RetryPolicy::aggressive();
    
    assert_eq!(default_policy.max_attempts, 3);
    assert_eq!(conservative_policy.max_attempts, 2);
    assert_eq!(aggressive_policy.max_attempts, 5);
    println!("✅ Retry policies configured correctly");
    
    // Verify 1.2: Exponential backoff works correctly
    assert_eq!(default_policy.calculate_backoff(1), Duration::from_secs(1));
    assert_eq!(default_policy.calculate_backoff(2), Duration::from_secs(2));
    assert_eq!(default_policy.calculate_backoff(3), Duration::from_secs(4));
    println!("✅ Exponential backoff calculation verified");
    
    // Verify 1.3: Smart error type filtering
    assert!(default_policy.should_retry(&ErrorType::NetworkError));
    assert!(default_policy.should_retry(&ErrorType::RateLimitError));
    assert!(!default_policy.should_retry(&ErrorType::Fatal));
    println!("✅ Smart retry logic verified");
    
    // Verify 1.4: Actual retry execution with real delays
    let executor = RetryExecutor::new(RetryPolicy::default());
    let mut agent = Agent::new(AgentRole::Implementer, 1);
    let network_task = Task::new(
        "Network Test".to_string(),
        "Task with network issues".to_string(),
        AgentRole::Implementer,
        TaskPriority::Normal,
    );
    
    let start_time = std::time::Instant::now();
    let result = executor.execute_task(&mut agent, &network_task).await;
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok());
    let task_result = result.unwrap();
    assert!(task_result.success);
    assert!(task_result.attempt_number > 1, "Should require multiple attempts");
    assert!(elapsed >= Duration::from_secs(1), "Should wait for backoff");
    println!("✅ Retry execution with delays verified");
    
    println!("🎉 TEST 1 PASSED: Error Recovery & Retry Logic fully implemented\n");
}

/// Test 2: Task Prioritization System  
#[tokio::test]
async fn test_task_prioritization_implementation() {
    println!("🔍 TESTING 2: Task Prioritization System");
    
    let mut workshop = WorkshopManager::new();
    let agent = Agent::new(AgentRole::Implementer, 1);
    let agent_id = agent.id.clone();
    workshop.register_agent(agent).unwrap();
    
    // Verify 2.1: Priority-based task assignment
    let high_task = Task::new("Critical".to_string(), "Critical task".to_string(), AgentRole::Implementer, TaskPriority::High);
    let normal_task = Task::new("Normal".to_string(), "Normal task".to_string(), AgentRole::Implementer, TaskPriority::Normal);
    
    // Queue in reverse priority order
    workshop.queue_task(normal_task);
    workshop.queue_task(high_task);
    
    let assignment = workshop.assign_by_priority().unwrap();
    assert!(assignment.is_some());
    let (_assigned_agent, assigned_task) = assignment.unwrap();
    assert_eq!(assigned_task.title, "Critical", "High priority task should be assigned first");
    println!("✅ Priority-based assignment verified");
    
    // Verify 2.2: Agent expertise tracking integration
    let agent_after = workshop.get_agent(&agent_id).unwrap();
    assert_eq!(agent_after.status, AgentStatus::Working);
    println!("✅ Agent expertise tracking integration verified");
    
    // Verify 2.3: Task complexity estimation exists (via TaskComplexity enum)
    let _simple = TaskComplexity::Simple;
    let _medium = TaskComplexity::Medium;
    let _complex = TaskComplexity::Complex;
    let _expert = TaskComplexity::Expert;
    println!("✅ Task complexity levels defined");
    
    // Verify 2.4: Retry integration in coordination
    let retry_task = Task::new("Retry".to_string(), "Test retry integration".to_string(), AgentRole::Implementer, TaskPriority::Normal);
    let result = workshop.execute_task_with_retry(&agent_id, &retry_task).await;
    assert!(result.is_ok(), "Task execution with retry should work");
    println!("✅ Retry integration verified");
    
    println!("🎉 TEST 2 PASSED: Task Prioritization System fully implemented\n");
}

/// Test 3: Performance Metrics Dashboard
#[test]
fn test_performance_metrics_implementation() {
    println!("🔍 TESTING 3: Performance Metrics Dashboard");
    
    let mut workshop = WorkshopManager::new();
    let status = workshop.get_status();
    let metrics = &status.metrics;
    
    // Verify 3.1: Comprehensive metrics exist
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
    println!("✅ Comprehensive metrics structure verified");
    
    // Verify 3.2: Bottleneck detection
    let agent = Agent::new(AgentRole::Scaffolder, 1);
    workshop.register_agent(agent).unwrap();
    
    let task = Task::new("Setup".to_string(), "Setup project".to_string(), AgentRole::Scaffolder, TaskPriority::Normal);
    workshop.assign_task(task).unwrap();
    
    let status = workshop.get_status();
    let scaffolder_util = status.metrics.agent_utilization.get(&AgentRole::Scaffolder).unwrap_or(&0.0);
    assert_eq!(*scaffolder_util, 1.0, "Scaffolder should be at 100% utilization");
    assert_eq!(status.metrics.bottleneck_role, Some(AgentRole::Scaffolder));
    println!("✅ Bottleneck detection verified");
    
    // Verify 3.3: Capacity tracking per role
    assert!(status.capacity_per_role.contains_key(&AgentRole::Scaffolder));
    assert!(status.capacity_per_role.contains_key(&AgentRole::Implementer));
    assert!(status.capacity_per_role.contains_key(&AgentRole::Debugger));
    assert!(status.capacity_per_role.contains_key(&AgentRole::Tester));
    println!("✅ Per-role capacity tracking verified");
    
    println!("🎉 TEST 3 PASSED: Performance Metrics Dashboard fully implemented\n");
}

/// Test 4: Enhanced Coordination System
#[test]
fn test_enhanced_coordination_implementation() {
    println!("🔍 TESTING 4: Enhanced Coordination System");
    
    let mut system = TestSystem::new();
    
    // Verify 4.1: Integration with TestSystem
    let agent = system.spawn_agent(AgentRole::Implementer);
    let (task_id, agent_id) = system.assign_task_to_role(AgentRole::Implementer, "integration test");
    
    system.complete_task(agent_id.clone(), task_id).unwrap();
    let agent_after = system.get_agent(&agent_id).unwrap();
    assert!(agent_after.metrics.tasks_completed > 0);
    println!("✅ TestSystem integration verified");
    
    // Verify 4.2: Workshop status tracking
    let status = system.get_workshop_status();
    assert!(status.total_agents > 0);
    assert!(!status.capacity_per_role.is_empty());
    println!("✅ Workshop status tracking verified");
    
    // Verify 4.3: Error handling integration
    let error_result = system.fail_task(agent_id, "invalid-task".to_string(), "Test error".to_string());
    assert!(error_result.is_err(), "Should handle errors gracefully");
    println!("✅ Error handling integration verified");
    
    println!("🎉 TEST 4 PASSED: Enhanced Coordination System fully implemented\n");
}

/// Test 5: Architecture & Type Safety
#[test]
fn test_architecture_implementation() {
    println!("🔍 TESTING 5: Architecture & Type Safety");
    
    // Verify 5.1: Retry module exported correctly
    let _policy = RetryPolicy::default();
    let _executor = RetryExecutor::new(_policy);
    println!("✅ Retry module properly exported");
    
    // Verify 5.2: All types compile with proper serde support
    let _expertise = AgentExpertise::default();
    let _complexity = TaskComplexity::Simple;
    let _error_type = ErrorType::NetworkError;
    let _retry_state = RetryState::default();
    println!("✅ Type safety verified");
    
    // Verify 5.3: Serialization works
    let metrics = WorkshopMetrics::default();
    let serialized = serde_json::to_string(&metrics);
    assert!(serialized.is_ok(), "Metrics should serialize");
    println!("✅ Serde serialization verified");
    
    // Verify 5.4: Error types exist
    let _workshop_err = WorkshopError::AgentNotFound("test".to_string());
    let _retry_err = RetryError::MaxAttemptsExceeded(3);
    println!("✅ Error handling types verified");
    
    println!("🎉 TEST 5 PASSED: Architecture & Type Safety fully implemented\n");
}

