//! Tests for the role-based agent system
//! 
//! Verifies that agents have correct role-specific behaviors and prompts.

use dfcoder_core::*;

#[tokio::test]
async fn test_role_specific_system_prompts() {
    println!("🧪 Testing Role-Specific System Prompts");
    
    let scaffolder = Agent::new(AgentRole::Scaffolder, 1);
    let implementer = Agent::new(AgentRole::Implementer, 2);
    let debugger = Agent::new(AgentRole::Debugger, 3);
    let tester = Agent::new(AgentRole::Tester, 4);
    
    // Test that each role has appropriate system prompt
    let scaffolder_prompt = scaffolder.system_prompt();
    assert!(scaffolder_prompt.contains("Scaffolder"));
    assert!(scaffolder_prompt.contains("project structure"));
    assert!(scaffolder_prompt.contains("boilerplate"));
    
    let implementer_prompt = implementer.system_prompt();
    assert!(implementer_prompt.contains("Implementer"));
    assert!(implementer_prompt.contains("features"));
    assert!(implementer_prompt.contains("business logic"));
    
    let debugger_prompt = debugger.system_prompt();
    assert!(debugger_prompt.contains("Debugger"));
    assert!(debugger_prompt.contains("bugs"));
    assert!(debugger_prompt.contains("error"));
    
    let tester_prompt = tester.system_prompt();
    assert!(tester_prompt.contains("Tester"));
    assert!(tester_prompt.contains("test"));
    assert!(tester_prompt.contains("coverage"));
    
    println!("✅ All role-specific prompts contain appropriate keywords");
    
    // Verify prompts are different
    assert_ne!(scaffolder_prompt, implementer_prompt);
    assert_ne!(implementer_prompt, debugger_prompt);
    assert_ne!(debugger_prompt, tester_prompt);
    
    println!("✅ All prompts are unique");
    
    println!("🎉 Role-Specific System Prompts Test PASSED");
}

#[tokio::test]
async fn test_agent_lifecycle() {
    println!("🧪 Testing Agent Lifecycle");
    
    let mut agent = Agent::new(AgentRole::Implementer, 1);
    
    // Test initial state
    assert_eq!(agent.status, AgentStatus::Idle);
    assert_eq!(agent.current_task, None);
    assert_eq!(agent.metrics.tasks_completed, 0);
    
    println!("✅ Agent created in correct initial state");
    
    // Test task assignment
    let task_id = "test-task-123".to_string();
    assert!(agent.assign_task(task_id.clone()).is_ok());
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(agent.current_task, Some(task_id.clone()));
    
    println!("✅ Task assignment working");
    
    // Test task completion
    let completed_task = agent.complete_task().unwrap();
    assert_eq!(completed_task, task_id);
    assert_eq!(agent.status, AgentStatus::Idle);
    assert_eq!(agent.current_task, None);
    assert_eq!(agent.metrics.tasks_completed, 1);
    
    println!("✅ Task completion working");
    
    println!("🎉 Agent Lifecycle Test PASSED");
}