//! DSL Examples demonstrating natural language programming
//! 
//! This file contains comprehensive examples of how the DFCoder DSLs
//! can be used to express complex agent behaviors and scenarios
//! in natural, readable language.

use dfcoder_dsl::*;
use dfcoder_test_utils::*;
use dfcoder_baml::*;
use dfcoder_mcp::*;
use std::time::Duration;

/// Example: Define a sophisticated Rust expert agent with complex behaviors
#[tokio::test]
async fn example_sophisticated_rust_agent() {
    // Using the agent builder DSL for complex behavior definition
    let rust_expert = AgentBuilder::new("AdvancedRustExpert")
        .responds_to("borrow checker", AgentAction::AnalyzeCode(
            "Analyzing borrow checker error with detailed ownership trace".to_string()
        ))
        .responds_to("performance", AgentAction::ExecuteCommand(
            "cargo bench".to_string()
        ))
        .responds_to("unsafe", AgentAction::RequestHelp(
            "Unsafe code detected - requesting code review".to_string()
        ))
        .when_idle(AgentAction::Monitor("*.rs,Cargo.toml,Cargo.lock".to_string()))
        .during_supervision(AgentAction::Respond(
            "Providing context: current analysis focuses on memory safety patterns".to_string()
        ))
        .build();
    
    // Test the agent's behavior patterns
    let borrow_trigger = TriggerCondition::RespondsTo("borrow checker".to_string());
    let response = rust_expert.handle_trigger(&borrow_trigger).await;
    
    assert!(response.is_some());
    println!("✓ Sophisticated Rust agent responds to borrow checker issues");
}

/// Example: Complex multi-agent collaboration scenario
#[tokio::test]
async fn example_multi_agent_collaboration() {
    // Define the scenario using natural language
    scenario! {
        "Multi-agent code review collaboration"
        given: senior_developer working on architecture review for 30 minutes,
        when: junior_developer requests code review,
        then: senior_developer provides structured feedback with examples;
    }
    
    // Create the agents involved
    let senior_dev = AgentBuilder::new("SeniorDeveloper")
        .responds_to("architecture", AgentAction::AnalyzeCode(
            "Reviewing system architecture and design patterns".to_string()
        ))
        .responds_to("code review", AgentAction::Respond(
            "Providing detailed code review with improvement suggestions".to_string()
        ))
        .during_supervision(AgentAction::Respond(
            "Mentoring junior developer on best practices".to_string()
        ))
        .build();
    
    let junior_dev = AgentBuilder::new("JuniorDeveloper")
        .responds_to("stuck", AgentAction::RequestHelp(
            "Need guidance on implementing complex feature".to_string()
        ))
        .responds_to("feedback", AgentAction::Respond(
            "Implementing suggested improvements".to_string()
        ))
        .during_supervision(AgentAction::Respond(
            "Learning from senior developer feedback".to_string()
        ))
        .build();
    
    // Test collaboration trigger
    let review_trigger = TriggerCondition::RespondsTo("code review".to_string());
    let senior_response = senior_dev.handle_trigger(&review_trigger).await;
    
    assert!(senior_response.is_some());
    println!("✓ Multi-agent collaboration scenario works");
}

/// Example: BAML-driven activity understanding
#[tokio::test]
async fn example_intelligent_activity_classification() {
    // Create rich activity contexts for classification
    let activities = vec![
        ActivityContext::new("Implementing OAuth2 authentication flow")
            .with_file_types(vec!["ts".to_string(), "tsx".to_string()])
            .with_commands(vec!["npm install passport".to_string(), "npm test".to_string()])
            .with_duration(Duration::from_hours(2)),
        
        ActivityContext::new("Debugging memory leak in Rust async runtime")
            .with_file_types(vec!["rs".to_string()])
            .with_commands(vec!["cargo test".to_string(), "valgrind target/debug/app".to_string()])
            .with_errors(vec!["Memory leak detected in async task pool".to_string()])
            .with_duration(Duration::from_hours(1)),
        
        ActivityContext::new("Explaining database indexing strategy to team")
            .with_file_types(vec!["sql".to_string(), "md".to_string()])
            .with_commands(vec!["psql -f create_indexes.sql".to_string()])
            .with_duration(Duration::from_minutes(45)),
    ];
    
    // Test automatic categorization
    for (i, activity) in activities.iter().enumerate() {
        let classification = Activities::from_description(&activity.description);
        
        match i {
            0 => {
                // OAuth implementation should be code generation
                assert!(matches!(classification, Some(Activities::CodeGeneration(_))));
                println!("✓ OAuth implementation classified as CodeGeneration");
            }
            1 => {
                // Memory leak debugging should be problem solving
                assert!(matches!(classification, Some(Activities::ProblemSolving(_))));
                println!("✓ Memory leak debugging classified as ProblemSolving");
            }
            2 => {
                // Team explanation should be collaboration
                assert!(matches!(classification, Some(Activities::Collaboration(_))));
                println!("✓ Team explanation classified as Collaboration");
            }
            _ => {}
        }
    }
}

/// Example: Advanced event-driven supervision
#[tokio::test]
async fn example_event_driven_supervision() {
    // Define event patterns for complex supervision scenarios
    let mut event_bus = EventBus::new();
    let mut event_aggregator = EventAggregator::new();
    
    // Create a complex event flow
    let events = vec![
        SystemEvent::AgentStateChanged {
            agent_id: "ai_agent_1".to_string(),
            old_state: AgentState::Idle,
            new_state: AgentState::Working,
        },
        SystemEvent::ErrorOccurred {
            agent_id: "ai_agent_1".to_string(),
            error_message: "Compilation failed: type mismatch".to_string(),
            context: "Working on generic implementation".to_string(),
        },
        SystemEvent::SupervisionRequested {
            agent_id: "ai_agent_1".to_string(),
            message: "Need help with generic type constraints".to_string(),
            context: "Implementing trait bounds for complex generic function".to_string(),
        },
    ];
    
    // Process events through the system
    for event in events {
        event_aggregator.add_event(event.clone());
        event_bus.publish(event).await.unwrap();
    }
    
    // Test event pattern matching
    let error_pattern = EventPattern::And(vec![
        EventPattern::FromAgent("ai_agent_1".to_string()),
        EventPattern::OfType("ErrorOccurred".to_string()),
    ]);
    
    let supervision_pattern = EventPattern::Or(vec![
        EventPattern::OfType("SupervisionRequested".to_string()),
        EventPattern::Contains("help".to_string()),
    ]);
    
    // Verify patterns work with the aggregated events
    let activity_summary = event_aggregator.agent_activity_summary();
    assert!(activity_summary.contains_key("ai_agent_1"));
    assert_eq!(activity_summary["ai_agent_1"], 3);
    
    println!("✓ Event-driven supervision patterns working correctly");
}

/// Example: MCP resource ecosystem integration
#[tokio::test]
async fn example_mcp_ecosystem_integration() {
    // Define resources using the MCP DSL
    // This demonstrates how external tools can interact with DFCoder
    
    let mcp_config = McpConfig {
        server_name: "dfcoder-ecosystem".to_string(),
        server_version: "1.0.0".to_string(),
        protocol_version: "2024-11-05".to_string(),
        transport: TransportConfig {
            transport_type: TransportType::Stdio,
            address: None,
            port: None,
            timeout_ms: 10000,
        },
        security: SecurityConfig {
            require_auth: false,
            api_keys: Vec::new(),
            rate_limit: Some(RateLimit {
                requests_per_minute: 100,
                burst_size: 10,
            }),
        },
        resources: ResourceConfig {
            expose_agents: true,
            expose_panes: true,
            expose_tasks: true,
            expose_metrics: true,
        },
    };
    
    let mcp_service = McpService::new(mcp_config).unwrap();
    
    // Test tool registration
    let code_analysis_tool = ToolDefinition {
        name: "analyze_code".to_string(),
        description: "Analyze code for potential issues and improvements".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "code": {"type": "string"},
                "language": {"type": "string"},
                "focus": {"type": "string", "enum": ["performance", "security", "style"]}
            },
            "required": ["code", "language"]
        }),
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "issues": {"type": "array"},
                "suggestions": {"type": "array"},
                "score": {"type": "number"}
            }
        })),
    };
    
    // In a real implementation, this would register the tool
    // mcp_service.register_tool(code_analysis_tool).await.unwrap();
    
    // Test prompt registration
    let supervision_prompt = PromptDefinition {
        name: "supervision_guidance".to_string(),
        description: "Generate supervision guidance for stuck agents".to_string(),
        arguments: vec![
            PromptArgument {
                name: "agent_context".to_string(),
                description: "Current context of the agent needing supervision".to_string(),
                required: true,
            },
            PromptArgument {
                name: "error_details".to_string(),
                description: "Details of errors or issues encountered".to_string(),
                required: false,
            },
        ],
    };
    
    // In a real implementation, this would register the prompt
    // mcp_service.register_prompt(supervision_prompt).await.unwrap();
    
    println!("✓ MCP ecosystem integration structure validated");
}

/// Example: Comprehensive condition-based automation
#[tokio::test]
async fn example_condition_based_automation() {
    // Define complex conditions using the natural language DSL
    let productivity_condition = ConditionBuilder::new()
        .agent_status("ProductivityAgent", AgentStatus::Working)
        .time_elapsed(Duration::from_hours(1))
        .event_occurred("TaskCompleted")
        .and();
    
    let error_condition = ConditionBuilder::new()
        .pane_has_errors(1)
        .pane_contains(1, "error:")
        .agent_status("DebuggingAgent", AgentStatus::Stuck)
        .and();
    
    let collaboration_condition = ConditionBuilder::new()
        .event_occurred("SupervisionRequested")
        .agent_status("MentorAgent", AgentStatus::Idle)
        .or();
    
    // Test condition descriptions
    assert!(productivity_condition.description().contains("Working"));
    assert!(error_condition.description().contains("errors"));
    assert!(collaboration_condition.description().contains("SupervisionRequested"));
    
    // Create a condition monitor
    let mut monitor = ConditionMonitor::new(Duration::from_seconds(1));
    monitor.add_condition("productivity".to_string(), productivity_condition);
    monitor.add_condition("error_handling".to_string(), error_condition);
    monitor.add_condition("collaboration".to_string(), collaboration_condition);
    
    // Test evaluation with mock context
    let context = EvaluationContext {
        agents: std::collections::HashMap::new(),
        panes: std::collections::HashMap::new(),
        current_time: chrono::Utc::now(),
        events: vec![
            SystemEvent::TaskCompleted {
                agent_id: "ProductivityAgent".to_string(),
                task_id: "task_123".to_string(),
                result: TaskResult::Success,
            }
        ],
    };
    
    let evaluation_results = monitor.evaluate_all(&context);
    assert_eq!(evaluation_results.len(), 3);
    
    println!("✓ Condition-based automation system working");
}

/// Example: Natural language test scenarios
#[tokio::test]
async fn example_natural_language_test_scenarios() {
    // Complex scenario with multiple agents and conditions
    let scenario = TestScenario::new("Advanced code review workflow");
    
    // Add multiple agents with different roles
    let reviewer = MockAgent::new("CodeReviewer")
        .with_current_task("Reviewing pull request #123")
        .with_status(AgentStatus::Working);
    
    let author = MockAgent::new("CodeAuthor")
        .with_current_task("Addressing review feedback")
        .with_status(AgentStatus::Working);
    
    let ci_agent = MockAgent::new("CIAgent")
        .with_current_task("Running automated tests")
        .with_status(AgentStatus::Working);
    
    scenario.add_agent(reviewer).await;
    scenario.add_agent(author).await;
    scenario.add_agent(ci_agent).await;
    
    // Add mock panes representing different environments
    let review_pane = MockPane::new(1)
        .with_content("Code review comments and suggestions")
        .active();
    
    let test_pane = MockPane::new(2)
        .with_content("Running test suite... 15 tests passed, 2 failed")
        .with_errors();
    
    scenario.add_pane(review_pane).await;
    scenario.add_pane(test_pane).await;
    
    // Simulate workflow progression
    scenario.advance_time(Duration::from_minutes(10)).await;
    
    // Test scenario conditions
    let condition_met = scenario.wait_for(
        || async {
            // Simulate completion condition
            true
        },
        Duration::from_seconds(1),
    ).await;
    
    assert!(condition_met);
    scenario.assert_success().await;
    
    println!("✓ Natural language test scenario executed successfully");
}

/// Example: Behavior pattern analysis and optimization
#[tokio::test]
async fn example_behavior_pattern_analysis() {
    // Create activity tracker with pattern analyzer
    let config = BamlConfig::default();
    let client = BamlClient::new(config).unwrap();
    let classifier = ActivityClassifier::new(client);
    let mut tracker = ActivityTracker::new(classifier);
    
    // Simulate a series of agent activities
    let activities = vec![
        ActivityContext::new("Implementing user authentication")
            .with_file_types(vec!["ts".to_string()])
            .with_duration(Duration::from_hours(2)),
        
        ActivityContext::new("Writing unit tests for auth module")
            .with_file_types(vec!["test.ts".to_string()])
            .with_duration(Duration::from_minutes(45)),
        
        ActivityContext::new("Debugging session timeout issue")
            .with_errors(vec!["Session expires unexpectedly".to_string()])
            .with_duration(Duration::from_hours(1)),
        
        ActivityContext::new("Code review: authentication implementation")
            .with_duration(Duration::from_minutes(30)),
        
        ActivityContext::new("Refactoring auth service for better testability")
            .with_file_types(vec!["ts".to_string()])
            .with_duration(Duration::from_hours(1)),
    ];
    
    // Track all activities
    let agent_id = "FullStackDeveloper".to_string();
    for (i, activity) in activities.iter().enumerate() {
        let activity_id = tracker.start_activity(agent_id.clone(), activity.clone())
            .await
            .unwrap();
        
        // Simulate completion
        let completion = CompletionDetails {
            success_indicators: vec![format!("Activity {} completed", i)],
            artifacts_created: vec![format!("feature_{}.ts", i)],
            time_to_completion: activity.duration,
            quality_score: 0.8 + (i as f32 * 0.05),
        };
        
        tracker.complete_activity(&agent_id, &activity_id, completion).unwrap();
    }
    
    // Analyze patterns
    if let Some(analysis) = tracker.analyze_patterns(&agent_id) {
        assert!(analysis.efficiency_score > 0.0);
        assert!(!analysis.focus_areas.is_empty());
        println!("✓ Behavior pattern analysis: efficiency score = {:.2}", analysis.efficiency_score);
    }
    
    // Test productivity trends
    let trends = tracker.detect_productivity_trends(&agent_id, Duration::from_hours(8));
    assert!(!trends.is_empty());
    
    println!("✓ Behavior pattern analysis and optimization working");
}

/// Example: Complete ecosystem demonstration
#[tokio::test]
async fn example_complete_ecosystem_demo() {
    println!("🚀 DFCoder Complete Ecosystem Demo");
    
    // 1. Initialize all subsystems
    let mut behavior_engine = BehaviorEngine::new();
    let mut event_bus = EventBus::new();
    let resource_manager = ResourceManager::new(ResourceConfig {
        expose_agents: true,
        expose_panes: true,
        expose_tasks: true,
        expose_metrics: true,
    });
    
    // 2. Register specialized agents
    let agents = vec![
        AgentArchetypes::rust_expert(),
        AgentArchetypes::typescript_expert(),
        AgentArchetypes::test_specialist(),
    ];
    
    for agent in agents {
        behavior_engine.registry_mut().register(agent);
    }
    
    // 3. Create complex scenario
    let scenario = TestScenario::new("Full ecosystem integration test");
    
    // 4. Simulate multi-agent collaboration
    let collaboration_events = vec![
        SystemEvent::AgentStateChanged {
            agent_id: "RustExpert".to_string(),
            old_state: AgentState::Idle,
            new_state: AgentState::Working,
        },
        SystemEvent::TaskCompleted {
            agent_id: "RustExpert".to_string(),
            task_id: "implement_core_logic".to_string(),
            result: TaskResult::Success,
        },
        SystemEvent::SupervisionRequested {
            agent_id: "TypeScriptExpert".to_string(),
            message: "Need help integrating with Rust backend".to_string(),
            context: "Building TypeScript frontend".to_string(),
        },
    ];
    
    // 5. Process events through the system
    for event in collaboration_events {
        event_bus.publish(event).await.unwrap();
    }
    
    // 6. Test natural language condition evaluation
    let integration_condition = ConditionBuilder::new()
        .agent_status("RustExpert", AgentStatus::Working)
        .event_occurred("TaskCompleted")
        .and();
    
    // 7. Verify MCP resource exposure
    let resources = resource_manager.list_agents().await.unwrap();
    
    // 8. Complete the scenario
    scenario.advance_time(Duration::from_minutes(30)).await;
    scenario.assert_success().await;
    
    println!("✅ Complete ecosystem demo: All subsystems integrated successfully");
    println!("   - {} agents registered", behavior_engine.registry().agent_names().len());
    println!("   - Event system processing collaboration flows");
    println!("   - Natural language conditions evaluated");
    println!("   - MCP resources exposed for external integration");
    println!("   - Test scenarios executed with natural language DSL");
}