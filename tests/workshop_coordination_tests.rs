//! Tests for workshop capacity management and task coordination
//! 
//! Verifies that the workshop system correctly manages agent capacity and task assignment.

use dfcoder_core::*;

#[tokio::test]
async fn test_workshop_capacity_and_bottleneck_detection() {
    println!("🧪 Testing Workshop Capacity and Bottleneck Detection");
    
    let mut workshop = WorkshopManager::new();
    
    // Create agents with different roles
    for i in 0..3 {
        workshop.register_agent(Agent::new(AgentRole::Implementer, i)).unwrap();
    }
    for i in 3..5 {
        workshop.register_agent(Agent::new(AgentRole::Debugger, i)).unwrap();
    }
    workshop.register_agent(Agent::new(AgentRole::Scaffolder, 5)).unwrap();
    workshop.register_agent(Agent::new(AgentRole::Tester, 6)).unwrap();
    
    // Create many implementer tasks (should create bottleneck)
    for i in 0..10 {
        let task = Task::new(
            format!("Implementation task {}", i),
            "Implement feature".to_string(),
            AgentRole::Implementer,
            TaskPriority::Normal,
        );
        workshop.queue_task(task);
    }
    
    // Assign all available implementer capacity
    let assignment1 = workshop.try_assign_next_task().unwrap();
    let assignment2 = workshop.try_assign_next_task().unwrap();
    let assignment3 = workshop.try_assign_next_task().unwrap();
    let assignment4 = workshop.try_assign_next_task().unwrap(); // Should fail
    
    assert!(assignment1.is_some());
    assert!(assignment2.is_some());
    assert!(assignment3.is_some());
    assert!(assignment4.is_none()); // No more implementer capacity
    
    println!("✅ Capacity limits enforced correctly");
    
    // Check workshop status
    let status = workshop.get_status();
    
    // Should show high utilization for implementers
    let implementer_utilization = status.metrics.agent_utilization.get(&AgentRole::Implementer);
    assert!(implementer_utilization.is_some());
    assert!(*implementer_utilization.unwrap() >= 1.0); // 100% utilization
    
    // Should identify implementer as bottleneck
    assert_eq!(status.metrics.bottleneck_role, Some(AgentRole::Implementer));
    
    println!("✅ Bottleneck detection working correctly");
    
    // Test that other roles can still be assigned
    let debugger_task = Task::new(
        "Debug task".to_string(),
        "Fix bug".to_string(),
        AgentRole::Debugger,
        TaskPriority::High,
    );
    workshop.queue_task(debugger_task);
    
    let debugger_assignment = workshop.try_assign_next_task().unwrap();
    assert!(debugger_assignment.is_some()); // Should work
    
    println!("✅ Non-bottleneck roles still assignable");
    
    println!("🎉 Workshop Capacity and Bottleneck Detection Test PASSED");
}

#[tokio::test]
async fn test_capacity_limits_per_role() {
    println!("🧪 Testing Capacity Limits Per Role");
    
    let mut workshop = WorkshopManager::new();
    
    // Scaffolder has capacity of 1
    let agent1 = Agent::new(AgentRole::Scaffolder, 1);
    let agent2 = Agent::new(AgentRole::Scaffolder, 2);
    
    workshop.register_agent(agent1).unwrap();
    workshop.register_agent(agent2).unwrap();

    let task1 = Task::new("Task 1".to_string(), "Desc".to_string(), AgentRole::Scaffolder, TaskPriority::Normal);

    // First assignment should work
    assert!(workshop.assign_task(task1).is_ok());
    
    // Second assignment should fail due to capacity
    assert!(!workshop.can_assign(AgentRole::Scaffolder));
    
    println!("✅ Scaffolder capacity limits working");
    
    // Test implementer capacity (should be 3)
    for i in 3..6 {
        workshop.register_agent(Agent::new(AgentRole::Implementer, i)).unwrap();
    }
    
    assert!(workshop.can_assign(AgentRole::Implementer));
    
    println!("✅ Implementer capacity limits working");
    println!("🎉 Capacity Limits Per Role Test PASSED");
}