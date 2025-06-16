//! Test Framework Scenarios
//! 
//! These tests demonstrate the TestSystem API and verify that agents
//! behave correctly in various coordination and supervision scenarios.

use dfcoder_test_utils::TestSystem;
use dfcoder_core::*;

#[tokio::test]
async fn test_agent_requests_help_when_stuck() {
    println!("🧪 Testing Agent Requests Help When Stuck");
    
    let mut system = TestSystem::new();
    
    // Create agent and assign task
    let agent = system.spawn_agent(AgentRole::Implementer);
    let task_id = system.assign_task(agent.id.clone(), "implement user auth");
    
    println!("✅ Agent spawned and task assigned");
    
    // Simulate agent getting stuck
    system.simulate_output(agent.id.clone(), "Error: cannot resolve type");
    system.advance_time(std::time::Duration::from_secs(120));
    
    println!("✅ Simulated agent getting stuck");
    
    // Verify supervision triggered
    assert!(system.has_supervision_request(), "Expected supervision request to be generated");
    
    let request = system.get_supervision_request().unwrap();
    assert!(request.context.contains("stuck") || request.context.contains("supervision"), 
            "Expected supervision context to mention being stuck, got: {}", request.context);
    
    println!("✅ Supervision request generated correctly");
    println!("🎉 Agent Requests Help When Stuck Test PASSED");
}

#[tokio::test]
async fn test_workshop_capacity_management() {
    println!("🧪 Testing Workshop Capacity Management");
    
    let mut system = TestSystem::new();
    
    // Test scaffolder capacity (limit: 1)
    let scaffolder1 = system.spawn_agent(AgentRole::Scaffolder);
    let scaffolder2 = system.spawn_agent(AgentRole::Scaffolder);
    
    // Assign task to first scaffolder
    let (_task1, _assigned_agent) = system.assign_task_to_role(AgentRole::Scaffolder, "setup project structure");
    
    // Workshop should be at capacity for scaffolders
    assert!(system.is_at_capacity(AgentRole::Scaffolder), 
            "Expected scaffolder role to be at capacity");
    
    println!("✅ Scaffolder capacity limits working");
    
    // Test implementer capacity (limit: 3)
    let impl1 = system.spawn_agent(AgentRole::Implementer);
    let impl2 = system.spawn_agent(AgentRole::Implementer);
    let impl3 = system.spawn_agent(AgentRole::Implementer);
    
    // Should be able to assign 3 tasks
    let (_task2, _agent2) = system.assign_task_to_role(AgentRole::Implementer, "implement feature A");
    let (_task3, _agent3) = system.assign_task_to_role(AgentRole::Implementer, "implement feature B");
    let (_task4, _agent4) = system.assign_task_to_role(AgentRole::Implementer, "implement feature C");
    
    // Now implementers should be at capacity
    assert!(system.is_at_capacity(AgentRole::Implementer), 
            "Expected implementer role to be at capacity");
    
    println!("✅ Implementer capacity limits working");
    
    // But other roles should still be available
    assert!(!system.is_at_capacity(AgentRole::Debugger), 
            "Debugger role should not be at capacity");
    assert!(!system.is_at_capacity(AgentRole::Tester), 
            "Tester role should not be at capacity");
    
    println!("✅ Other roles still available");
    println!("🎉 Workshop Capacity Management Test PASSED");
}

#[tokio::test]
async fn test_agent_lifecycle_management() {
    println!("🧪 Testing Agent Lifecycle Management");
    
    let mut system = TestSystem::new();
    
    // Spawn agent
    let agent = system.spawn_agent(AgentRole::Debugger);
    let agent_ref = system.get_agent(&agent.id).unwrap();
    
    // Initially idle
    assert_eq!(agent_ref.status, AgentStatus::Idle);
    assert_eq!(agent_ref.metrics.tasks_completed, 0);
    
    println!("✅ Agent starts in idle state");
    
    // Assign task
    let (task_id, actual_agent_id) = system.assign_task_to_role(AgentRole::Debugger, "debug authentication issue");
    
    // Update our reference to track the actual assigned agent
    let actual_agent_id = actual_agent_id;
    
    // Should be working
    let agent_ref = system.get_agent(&actual_agent_id).unwrap();
    assert_eq!(agent_ref.status, AgentStatus::Working);
    assert!(agent_ref.current_task.is_some());
    
    println!("✅ Agent transitions to working state");
    
    // Complete task
    system.complete_task(actual_agent_id.clone(), task_id).unwrap();
    
    // Should be idle again with updated metrics
    let agent_ref = system.get_agent(&actual_agent_id).unwrap();
    assert_eq!(agent_ref.status, AgentStatus::Idle);
    assert_eq!(agent_ref.metrics.tasks_completed, 1);
    assert!(agent_ref.current_task.is_none());
    
    println!("✅ Agent completes task and returns to idle");
    println!("🎉 Agent Lifecycle Management Test PASSED");
}

#[tokio::test]
async fn test_supervision_dialogue_generation() {
    println!("🧪 Testing Supervision Dialogue Generation");
    
    let mut system = TestSystem::new();
    
    // Create stuck agent
    let agent = system.spawn_agent(AgentRole::Implementer);
    let _task_id = system.assign_task(agent.id.clone(), "complex OAuth integration");
    
    // Simulate different types of stuck scenarios
    system.simulate_output(agent.id.clone(), "Error: I'm completely stuck and confused, need help");
    system.advance_time(std::time::Duration::from_secs(30));
    
    // Should generate supervision request
    assert!(system.has_supervision_request(), "Expected supervision request");
    
    let request = system.get_supervision_request_for(&agent.id).unwrap();
    
    // Should have multiple dialogue options
    assert!(!request.options.is_empty(), "Expected dialogue options");
    assert!(request.options.len() >= 3, "Expected multiple dialogue options");
    
    // Should have appropriate action types
    let has_guidance = request.options.iter().any(|o| 
        matches!(o.action, SupervisionAction::ProvideGuidance(_)));
    let has_ignore = request.options.iter().any(|o| 
        matches!(o.action, SupervisionAction::IgnoreForNow));
    let has_escalate = request.options.iter().any(|o| 
        matches!(o.action, SupervisionAction::EscalateToHuman));
    
    assert!(has_guidance, "Expected guidance option");
    assert!(has_ignore, "Expected ignore option");
    assert!(has_escalate, "Expected escalate option");
    
    println!("✅ Supervision dialogue generated with appropriate options");
    
    // Test responding to supervision
    let guidance_option = request.options.iter()
        .find(|o| matches!(o.action, SupervisionAction::ProvideGuidance(_)))
        .unwrap();
    
    let action = system.respond_to_supervision(&agent.id, guidance_option.id).await.unwrap();
    
    match action {
        SupervisionAction::ProvideGuidance(guidance) => {
            assert!(!guidance.is_empty(), "Expected non-empty guidance");
            println!("✅ Received guidance: {}", guidance);
        }
        _ => panic!("Expected guidance action"),
    }
    
    // Should no longer have active supervision request
    assert!(!system.is_agent_stuck(&agent.id), "Agent should no longer be stuck");
    
    println!("✅ Supervision response handled correctly");
    println!("🎉 Supervision Dialogue Generation Test PASSED");
}

#[tokio::test]
async fn test_multi_agent_coordination() {
    println!("🧪 Testing Multi-Agent Coordination");
    
    let mut system = TestSystem::new();
    
    // Create a realistic workflow
    let scaffolder = system.spawn_agent(AgentRole::Scaffolder);
    let implementer = system.spawn_agent(AgentRole::Implementer);
    let debugger = system.spawn_agent(AgentRole::Debugger);
    let tester = system.spawn_agent(AgentRole::Tester);
    
    println!("✅ All agent roles spawned");
    
    // Assign tasks in order
    let (setup_task, setup_agent) = system.assign_task_to_role(AgentRole::Scaffolder, "setup project structure");
    let (_impl_task, _impl_agent) = system.assign_task_to_role(AgentRole::Implementer, "implement user service");
    let (_debug_task, _debug_agent) = system.assign_task_to_role(AgentRole::Debugger, "fix memory leak");
    let (_test_task, _test_agent) = system.assign_task_to_role(AgentRole::Tester, "write integration tests");
    
    println!("✅ Tasks assigned to all agents");
    
    // Check workshop status
    let status = system.get_workshop_status();
    assert_eq!(status.total_agents, 4);
    assert_eq!(status.active_agents, 4);
    assert_eq!(status.queue_length, 0); // All tasks assigned
    
    println!("✅ Workshop status reflects active agents");
    
    // Complete scaffolder task
    system.complete_task(setup_agent, setup_task).unwrap();
    
    // Status should update
    let status = system.get_workshop_status();
    assert_eq!(status.active_agents, 3); // One agent now idle
    
    println!("✅ Workshop status updates after task completion");
    
    // Simulate some agents finishing
    system.simulate_output(implementer.id.clone(), "Successfully implemented user service");
    system.simulate_output(tester.id.clone(), "All tests passing with 95% coverage");
    
    println!("✅ Agents completing work successfully");
    
    println!("🎉 Multi-Agent Coordination Test PASSED");
}

#[tokio::test] 
async fn test_system_integration_scenarios() {
    println!("🧪 Testing System Integration Scenarios");
    
    let mut system = TestSystem::new();
    
    // Complex scenario: Agent gets stuck, gets help, resolves issue
    let agent = system.spawn_agent(AgentRole::Implementer);
    let (task_id, actual_agent_id) = system.assign_task_to_role(AgentRole::Implementer, "implement complex algorithm");
    
    println!("✅ Test scenario setup complete");
    
    // Agent starts working
    system.simulate_output(actual_agent_id.clone(), "Starting implementation of sorting algorithm");
    system.advance_time(std::time::Duration::from_secs(60));
    
    // Agent encounters issue
    system.simulate_output(actual_agent_id.clone(), "Error: performance is too slow, tried multiple approaches");
    system.advance_time(std::time::Duration::from_secs(120));
    
    // Should trigger supervision
    assert!(system.has_supervision_request(), "Expected supervision request");
    
    println!("✅ Supervision triggered for stuck agent");
    
    // Supervisor provides guidance
    let request = system.get_supervision_request_for(&actual_agent_id).unwrap();
    let guidance_option = request.options.iter()
        .find(|o| matches!(o.action, SupervisionAction::ProvideGuidance(_)))
        .unwrap();
    
    let _action = system.respond_to_supervision(&actual_agent_id, guidance_option.id).await.unwrap();
    
    println!("✅ Supervision guidance provided");
    
    // Agent resolves issue
    system.simulate_output(actual_agent_id.clone(), "Thanks! Used the suggested approach and performance improved 10x");
    system.advance_time(std::time::Duration::from_secs(60));
    
    // Complete task successfully
    system.complete_task(actual_agent_id.clone(), task_id).unwrap();
    
    // Verify final state
    let agent_ref = system.get_agent(&actual_agent_id).unwrap();
    assert_eq!(agent_ref.status, AgentStatus::Idle);
    assert_eq!(agent_ref.metrics.tasks_completed, 1);
    assert!(!system.is_agent_stuck(&actual_agent_id));
    
    println!("✅ Agent successfully completed task after supervision");
    println!("🎉 System Integration Scenarios Test PASSED");
}