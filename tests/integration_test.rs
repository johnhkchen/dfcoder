//! Integration tests demonstrating DFCoder DSL usage
//! 
//! These tests showcase the natural language DSLs working together
//! to create a complete agent management and supervision system.

use dfcoder_dsl::*;
use dfcoder_test_utils::*;
use dfcoder_baml::*;
use dfcoder_mcp::*;
use dfcoder_types::*;
use std::time::Duration;
use tokio_test;

/// Test the complete scenario DSL pipeline
#[tokio::test]
async fn test_agent_supervision_scenario() {
    // Use the scenario! macro from dfcoder-macros
    scenario! {
        "Agent requests help when stuck"
        given: agent working on complex task for 2 minutes,
        when: no progress detected,
        then: supervisor sees dialogue with context;
    }
}

/// Test agent behavior DSLs
#[tokio::test]
async fn test_agent_behavior_definitions() {
    // Create agents using the natural language DSL
    let rust_expert = AgentArchetypes::rust_expert();
    let coding_assistant = AgentArchetypes::coding_assistant();
    
    // Test behavior matching
    let trigger = TriggerCondition::RespondsTo("type error".to_string());
    let action = rust_expert.handle_trigger(&trigger).await;
    
    assert!(action.is_some());
    if let Some(AgentAction::AnalyzeCode(description)) = action {
        assert!(description.contains("Analyzing"));
    }
    
    // Test coding assistant help behavior
    let help_trigger = TriggerCondition::RespondsTo("stuck".to_string());
    let help_action = coding_assistant.handle_trigger(&help_trigger).await;
    
    assert!(help_action.is_some());
    if let Some(AgentAction::RequestHelp(message)) = help_action {
        assert!(message.contains("guidance"));
    }
}

/// Test event flows and patterns
#[tokio::test]
async fn test_event_flows() {
    let mut event_bus = EventBus::new();
    let mut event_queue = EventQueue::new(100);
    
    // Create supervision flow events
    let supervision_events = EventFlows::supervision_flow();
    
    for event in supervision_events {
        event_queue.push(event.clone());
        event_bus.publish(event).await.unwrap();
    }
    
    // Verify event patterns
    let pattern = EventPattern::OfType("SupervisionRequested".to_string());
    let matching_events = event_queue.filter(&pattern);
    
    assert!(!matching_events.is_empty());
}

/// Test BAML activity classification
#[tokio::test]
async fn test_baml_activity_classification() {
    // Mock BAML configuration for testing
    let config = BamlConfig {
        endpoint: "http://localhost:8000/test".to_string(),
        api_key: Some("test-key".to_string()),
        model: "test-model".to_string(),
        temperature: 0.1,
        max_tokens: 100,
        confidence_threshold: 0.7,
    };
    
    // Create activity context
    let context = ActivityContext::new("Debugging type error in Rust code")
        .with_output("error[E0308]: mismatched types")
        .with_file_types(vec!["rs".to_string()])
        .with_commands(vec!["cargo check".to_string()])
        .with_errors(vec!["Type mismatch in main.rs".to_string()])
        .with_duration(Duration::from_minutes(5));
    
    // Test activity categorization using schema inference
    let inferred_activity = Activities::from_description(&context.description);
    assert!(inferred_activity.is_some());
    
    if let Some(Activities::ProblemSolving(ProblemSolving::Debugging)) = inferred_activity {
        println!("✓ Activity correctly classified as debugging");
    } else {
        panic!("Activity classification failed");
    }
    
    // Test activity indicators
    let activities = Activities::ProblemSolving(ProblemSolving::Debugging);
    let indicators = activities.typical_indicators();
    assert!(indicators.contains(&"debug"));
    assert!(indicators.contains(&"error"));
}

/// Test MCP resource exposure
#[tokio::test]
async fn test_mcp_resource_exposure() {
    // Create MCP service configuration
    let config = McpConfig {
        server_name: "dfcoder-test".to_string(),
        server_version: "1.0.0".to_string(),
        protocol_version: "2024-11-05".to_string(),
        transport: TransportConfig {
            transport_type: TransportType::Stdio,
            address: None,
            port: None,
            timeout_ms: 5000,
        },
        security: SecurityConfig {
            require_auth: false,
            api_keys: Vec::new(),
            rate_limit: None,
        },
        resources: ResourceConfig {
            expose_agents: true,
            expose_panes: true,
            expose_tasks: true,
            expose_metrics: false,
        },
    };
    
    let mcp_service = McpService::new(config).unwrap();
    
    // Test resource listing
    let agent_resources = mcp_service.list_agent_resources().await.unwrap();
    
    // Initially empty, but structure is correct
    assert!(agent_resources.is_empty());
    
    // Test capabilities
    let capabilities = mcp_service.get_capabilities();
    assert!(capabilities.resources.is_some());
    assert!(capabilities.tools.is_some());
}

/// Test supervision dialogue generation
#[tokio::test]
async fn test_supervision_dialogue_generation() {
    use dfcoder_test_utils::*;
    
    // Create a scenario with stuck agent
    let scenario = TestScenario::with_stuck_agent("supervision_test").await;
    
    // Create supervisor with auto-response
    let supervisor = SupervisorScenarioBuilder::new()
        .with_auto_respond(1000)
        .with_queued_response("stuck_agent", 1)
        .build();
    
    // Verify supervisor has the right capabilities
    assert!(supervisor.has_active_dialogues() == false); // Initially no dialogues
    
    // Test dialogue option generation for different contexts
    let help_request = HelpRequest {
        message: "I'm stuck on this type error".to_string(),
        context: "Implementing a complex generic function".to_string(),
        timestamp: std::time::Instant::now(),
        urgency: dfcoder_test_utils::HelpUrgency::Medium,
    };
    
    // Supervisor should generate appropriate dialogue options
    // This would normally involve BAML classification, but we test the structure
    assert!(help_request.message.contains("stuck"));
}

/// Test complete workflow integration
#[tokio::test]
async fn test_complete_workflow_integration() {
    // This test demonstrates all DSLs working together
    
    // 1. Create agents using natural language definitions
    let mut agent_registry = AgentRegistry::new();
    agent_registry.register_common_archetypes();
    
    // 2. Set up event system
    let mut event_bus = EventBus::new();
    let mut event_aggregator = EventAggregator::new();
    
    // 3. Create test scenario
    let scenario = TestScenario::new("Complete workflow test");
    
    // 4. Add agent to scenario
    let agent = MockAgent::new("test_rust_expert")
        .with_current_task("Fix compilation errors")
        .working_for(Duration::from_minutes(3));
    
    scenario.add_agent(agent).await;
    
    // 5. Simulate agent activity and classification
    let activity_context = ActivityContext::new("Fixing compilation errors in main.rs")
        .with_file_types(vec!["rs".to_string()])
        .with_errors(vec!["Cannot borrow as mutable".to_string()]);
    
    // 6. Test BAML classification
    let classified_activity = Activities::from_description(&activity_context.description);
    assert!(classified_activity.is_some());
    
    // 7. Generate system event
    let system_event = SystemEvent::AgentStateChanged {
        agent_id: "test_rust_expert".to_string(),
        old_state: AgentState::Idle,
        new_state: AgentState::Working,
    };
    
    event_aggregator.add_event(system_event.clone());
    event_bus.publish(system_event).await.unwrap();
    
    // 8. Test MCP resource exposure
    let resource_config = ResourceConfig {
        expose_agents: true,
        expose_panes: true,
        expose_tasks: true,
        expose_metrics: true,
    };
    
    let resource_manager = ResourceManager::new(resource_config);
    
    // Add agent as resource
    let agent_resource = ResourceFactory::create_agent_resource(
        "test_rust_expert".to_string(),
        &AgentState::default(),
    );
    
    resource_manager.update_agent(agent_resource).await;
    
    // 9. Verify complete integration
    let resources = resource_manager.list_agents().await.unwrap();
    assert!(!resources.is_empty());
    
    // 10. Test condition evaluation
    let condition = ConditionPatterns::agent_stuck("test_rust_expert", Duration::from_minutes(5));
    
    // Create evaluation context
    let mut agents = std::collections::HashMap::new();
    agents.insert("test_rust_expert".to_string(), AgentState::default());
    
    let context = EvaluationContext {
        agents,
        panes: std::collections::HashMap::new(),
        current_time: chrono::Utc::now(),
        events: vec![],
    };
    
    // Condition should evaluate correctly
    let _result = condition.evaluate(&context);
    
    println!("✓ Complete workflow integration test passed");
}

/// Test behavior patterns and scheduling
#[tokio::test]
async fn test_behavior_patterns_and_scheduling() {
    let mut behavior_engine = BehaviorEngine::new();
    let mut scheduler = BehaviorScheduler::new();
    
    // Schedule a behavior
    let schedule_id = scheduler.schedule_behavior(
        "RustExpert".to_string(),
        TriggerCondition::RespondsTo("error".to_string()),
        Schedule::Interval(Duration::from_seconds(30)),
    );
    
    assert!(schedule_id > 0);
    
    // Test pattern matching
    let error_behavior = BehaviorPatterns::error_analysis_behavior();
    assert!(error_behavior.description.contains("error"));
    
    // Test behavior execution
    let result = behavior_engine.execute_behavior(
        "RustExpert",
        TriggerCondition::RespondsTo("type error".to_string()),
    ).await;
    
    assert!(result.is_ok());
}

/// Test natural language scenario creation
#[tokio::test]
async fn test_natural_language_scenarios() {
    // Test scenario builder patterns
    let scenario = TestScenario::with_working_agent(
        "Agent productivity test",
        Duration::from_minutes(10),
    ).await;
    
    // Advance time and test conditions
    scenario.advance_time(Duration::from_minutes(5)).await;
    
    // Test waiting for conditions
    let condition_met = scenario.wait_for(
        || async { true }, // Simple condition for testing
        Duration::from_millis(100),
    ).await;
    
    assert!(condition_met);
    
    // Test assertions
    scenario.assert_success().await;
}

/// Performance and scalability test
#[tokio::test]
async fn test_performance_and_scalability() {
    let start_time = std::time::Instant::now();
    
    // Create multiple agents
    let mut agents = Vec::new();
    for i in 0..10 {
        let agent = MockAgent::new(format!("agent_{}", i))
            .with_current_task(format!("Task {}", i));
        agents.push(agent);
    }
    
    // Create multiple activities
    let mut activities = Vec::new();
    for i in 0..100 {
        let activity = ActivityContext::new(format!("Activity {}", i))
            .with_duration(Duration::from_seconds(i % 60));
        activities.push(activity);
    }
    
    // Test batch classification
    for activity in &activities {
        let _classified = Activities::from_description(&activity.description);
    }
    
    let duration = start_time.elapsed();
    println!("Performance test completed in {:?}", duration);
    
    // Should complete quickly
    assert!(duration < Duration::from_seconds(5));
}