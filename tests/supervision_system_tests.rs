//! Tests for the context-aware supervision system
//! 
//! Verifies that supervision requests are generated correctly and dialogue options are appropriate.

use dfcoder_core::*;
use dfcoder_baml::*;

#[tokio::test]
async fn test_supervision_system() {
    println!("🧪 Testing Context-Aware Supervision System");
    
    let mut supervision = SupervisionSystem::new();
    
    // 1. Test supervision request generation
    let agent = Agent::new(AgentRole::Implementer, 1);
    let stuck_output = "Error: I'm stuck and can't proceed";
    
    let request = supervision.check_supervision_need(&agent, stuck_output).await.unwrap();
    assert!(request.is_some());
    
    let req = request.unwrap();
    assert_eq!(req.agent_id, agent.id);
    assert!(!req.options.is_empty());
    assert!(req.context.contains("supervision"));
    
    println!("✅ Supervision request generation working");
    
    // 2. Test dialogue option generation with different scenarios
    
    // Stuck agent scenario
    let stuck_activity = ActivityClass {
        primary: ActivityType::Stuck,
        confidence: 0.2,
        needs_help: true,
        emotional_state: EmotionalState::Frustrated,
        estimated_completion: None,
    };
    
    let stuck_options = generate_dialogue_options(&agent, &stuck_activity, "I'm stuck");
    assert!(!stuck_options.is_empty());
    
    // Should have guidance and breakdown options
    assert!(stuck_options.iter().any(|o| 
        matches!(o.action, SupervisionAction::ProvideGuidance(_))));
    assert!(stuck_options.iter().any(|o| 
        matches!(o.action, SupervisionAction::BreakDownTask)));
    
    println!("✅ Stuck scenario dialogue options correct");
    
    // 3. Test urgency determination
    assert_eq!(determine_urgency(&stuck_activity), SupervisionUrgency::High);
    
    println!("✅ Urgency determination working correctly");
    
    // 4. Test supervision response handling
    let response = supervision.handle_supervision_response(&agent.id, 1).await;
    assert!(response.is_ok());
    
    // Should no longer have active request
    assert!(supervision.get_active_request(&agent.id).is_none());
    
    println!("✅ Supervision response handling working");
    
    println!("🎉 Supervision System Test PASSED");
}

#[tokio::test]
async fn test_urgency_determination() {
    println!("🧪 Testing Urgency Determination");
    
    let desperate = ActivityClass {
        primary: ActivityType::Stuck,
        confidence: 0.1,
        needs_help: true,
        emotional_state: EmotionalState::Desperate,
        estimated_completion: None,
    };
    assert_eq!(determine_urgency(&desperate), SupervisionUrgency::Critical);

    let frustrated_stuck = ActivityClass {
        primary: ActivityType::Stuck,
        confidence: 0.2,
        needs_help: true,
        emotional_state: EmotionalState::Frustrated,
        estimated_completion: None,
    };
    assert_eq!(determine_urgency(&frustrated_stuck), SupervisionUrgency::High);

    let low_confidence = ActivityClass {
        primary: ActivityType::Implementing,
        confidence: 0.2,  // Changed from 0.3 to 0.2 to trigger c < 0.3 condition
        needs_help: true,
        emotional_state: EmotionalState::Cautious,
        estimated_completion: None,
    };
    assert_eq!(determine_urgency(&low_confidence), SupervisionUrgency::Medium);
    
    println!("✅ All urgency levels determined correctly");
    println!("🎉 Urgency Determination Test PASSED");
}