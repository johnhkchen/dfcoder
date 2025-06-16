//! End-to-end integration tests
//! 
//! Tests complete workflows that span multiple components and verify they work together.

use dfcoder_core::*;
use dfcoder_baml::*;

#[tokio::test]
async fn test_complete_agent_workflow() {
    println!("🧪 Testing Complete Agent Workflow");
    
    // 1. Create workshop and register agents
    let mut workshop = WorkshopManager::new();
    
    // Create agents for each role
    let scaffolder = Agent::new(AgentRole::Scaffolder, 1);
    let implementer = Agent::new(AgentRole::Implementer, 2);
    let debugger = Agent::new(AgentRole::Debugger, 3);
    let tester = Agent::new(AgentRole::Tester, 4);
    
    let scaffolder_id = scaffolder.id.clone();
    
    workshop.register_agent(scaffolder).unwrap();
    workshop.register_agent(implementer).unwrap();
    workshop.register_agent(debugger).unwrap();
    workshop.register_agent(tester).unwrap();
    
    // Verify all agents registered
    assert_eq!(workshop.get_all_agents().len(), 4);
    println!("✅ All 4 agents registered successfully");
    
    // 2. Create tasks with dependencies
    let setup_task = Task::new(
        "Setup project structure".to_string(),
        "Create directories and config files".to_string(),
        AgentRole::Scaffolder,
        TaskPriority::High,
    );
    let setup_task_id = setup_task.id.clone();
    
    let implement_task = Task::new(
        "Implement user authentication".to_string(),
        "Add login and registration functionality".to_string(),
        AgentRole::Implementer,
        TaskPriority::Normal,
    );
    
    // Queue tasks
    workshop.queue_task(setup_task);
    workshop.queue_task(implement_task);
    
    assert_eq!(workshop.get_queue().len(), 2);
    println!("✅ Tasks queued successfully");
    
    // 3. Test task assignment
    let assignment1 = workshop.try_assign_next_task().unwrap();
    assert!(assignment1.is_some());
    let (assigned_agent_id, _) = assignment1.unwrap();
    
    // Should assign to scaffolder first (high priority setup task)
    assert_eq!(assigned_agent_id, scaffolder_id);
    
    println!("✅ Task assignment working correctly");
    
    // 4. Complete first task
    workshop.complete_task(scaffolder_id.clone(), setup_task_id).unwrap();
    
    // Verify agent is now idle
    let scaffolder_agent = workshop.get_agent(&scaffolder_id).unwrap();
    assert_eq!(scaffolder_agent.status, AgentStatus::Idle);
    assert_eq!(scaffolder_agent.metrics.tasks_completed, 1);
    
    println!("✅ Task completion working correctly");
    
    // 5. Test workshop status
    let status = workshop.get_status();
    assert_eq!(status.total_agents, 4);
    assert!(status.capacity_per_role.contains_key(&AgentRole::Scaffolder));
    
    println!("✅ Workshop status reporting working correctly");
    
    println!("🎉 Complete Agent Workflow Test PASSED");
}

#[tokio::test]
async fn test_end_to_end_agent_supervision_flow() {
    println!("🧪 Testing End-to-End Agent Supervision Flow");
    
    // This test demonstrates the complete flow from agent getting stuck to supervision resolution
    
    let mut workshop = WorkshopManager::new();
    let mut supervision = SupervisionSystem::new();
    
    // 1. Create and register agent
    let agent = Agent::new(AgentRole::Implementer, 1);
    let agent_id = agent.id.clone();
    workshop.register_agent(agent.clone()).unwrap();
    
    // 2. Create and assign a task
    let task = Task::new(
        "Complex feature implementation".to_string(),
        "Implement OAuth integration".to_string(),
        AgentRole::Implementer,
        TaskPriority::High,
    );
    let task_id = task.id.clone();
    workshop.queue_task(task);
    
    let assignment = workshop.try_assign_next_task().unwrap().unwrap();
    assert_eq!(assignment.0, agent_id);
    
    println!("✅ Agent assigned to complex task");
    
    // 3. Simulate agent getting stuck
    let stuck_output = "Error: OAuth library not found. I've tried multiple approaches but can't resolve this dependency issue.";
    
    // 4. Classify the agent's activity
    let activity_class = classify_activity(stuck_output).await;
    assert_eq!(activity_class.primary, ActivityType::Stuck);
    assert!(activity_class.needs_help);
    assert_eq!(activity_class.emotional_state, EmotionalState::Frustrated);
    
    println!("✅ BAML correctly classified agent as stuck and needing help");
    
    // 5. Generate supervision request
    let supervision_request = supervision.check_supervision_need(&agent, stuck_output).await.unwrap();
    assert!(supervision_request.is_some());
    
    let request = supervision_request.unwrap();
    assert_eq!(request.urgency, SupervisionUrgency::High);
    assert!(!request.options.is_empty());
    
    // Should have appropriate options for stuck agent
    let has_guidance = request.options.iter().any(|o| 
        matches!(o.action, SupervisionAction::ProvideGuidance(_)));
    
    assert!(has_guidance);
    
    println!("✅ Supervision request generated with appropriate options");
    
    // 6. Simulate supervisor choosing to provide guidance
    let guidance_option = request.options.iter()
        .find(|o| matches!(o.action, SupervisionAction::ProvideGuidance(_)))
        .unwrap();
    
    let action = supervision.handle_supervision_response(&agent_id, guidance_option.id).await.unwrap();
    
    match action {
        SupervisionAction::ProvideGuidance(guidance) => {
            assert!(guidance.contains("step by step") || guidance.contains("error"));
            println!("✅ Supervisor provided guidance: {}", guidance);
        }
        _ => panic!("Expected guidance action"),
    }
    
    // 7. Verify supervision request is resolved
    assert!(supervision.get_active_request(&agent_id).is_none());
    
    // 8. Simulate agent resolving the issue after guidance
    let resolved_output = "Thanks for the guidance! I found the issue - missing dependency in Cargo.toml. Adding it now.";
    
    let resolved_activity = classify_activity(resolved_output).await;
    assert_eq!(resolved_activity.primary, ActivityType::Implementing);
    assert!(!resolved_activity.needs_help);
    assert_eq!(resolved_activity.emotional_state, EmotionalState::Focused);
    
    println!("✅ Agent successfully resolved issue after supervision");
    
    // 9. Complete the task
    workshop.complete_task(agent_id.clone(), task_id).unwrap();
    
    let completed_agent = workshop.get_agent(&agent_id).unwrap();
    assert_eq!(completed_agent.status, AgentStatus::Idle);
    assert_eq!(completed_agent.metrics.tasks_completed, 1);
    
    println!("✅ Task completed successfully");
    
    println!("🎉 End-to-End Agent Supervision Flow Test PASSED");
}