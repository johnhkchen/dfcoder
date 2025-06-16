//! Tests for BAML activity classification system
//! 
//! Verifies that agent output is correctly classified into activity types and emotional states.

use dfcoder_baml::*;

#[tokio::test]
async fn test_baml_activity_classification() {
    println!("🧪 Testing BAML Activity Classification");
    
    // Test various activity types
    let test_cases = vec![
        ("Error: compilation failed", ActivityType::Stuck, true),
        ("Debugging the authentication error", ActivityType::Debugging, false),
        ("Running tests to verify functionality", ActivityType::Testing, false),
        ("Setting up project structure with cargo init", ActivityType::Scaffolding, false),
        ("Implementing user login feature", ActivityType::Implementing, false),
        ("Reading documentation about OAuth", ActivityType::Researching, false),
        ("Waiting for user input", ActivityType::Waiting, false),
        ("I'm completely stuck and need help", ActivityType::Stuck, true),
        ("Successfully completed the implementation", ActivityType::Implementing, false),
    ];
    
    for (output, expected_activity, expected_needs_help) in test_cases {
        let result = classify_activity(output).await;
        
        assert_eq!(result.primary, expected_activity, 
                   "Failed for input: '{}' - expected {:?}, got {:?}", 
                   output, expected_activity, result.primary);
        
        assert_eq!(result.needs_help, expected_needs_help,
                   "Failed needs_help for input: '{}' - expected {}, got {}", 
                   output, expected_needs_help, result.needs_help);
        
        // Verify confidence is reasonable
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        
        println!("✅ '{}' -> {:?} (needs_help: {})", 
                 output.chars().take(30).collect::<String>(), 
                 result.primary, result.needs_help);
    }
    
    println!("🎉 BAML Activity Classification Test PASSED");
}

#[tokio::test]
async fn test_emotional_state_detection() {
    println!("🧪 Testing Emotional State Detection");
    
    // Test emotional states
    let desperate_result = classify_activity("I'm stuck and confused, need help").await;
    assert_eq!(desperate_result.emotional_state, EmotionalState::Desperate);
    
    let confident_result = classify_activity("Everything working perfectly, completed successfully").await;
    assert_eq!(confident_result.emotional_state, EmotionalState::Confident);
    
    let frustrated_result = classify_activity("Error: failed again, this is not working").await;
    assert_eq!(frustrated_result.emotional_state, EmotionalState::Frustrated);
    
    println!("✅ Emotional state detection working correctly");
    println!("🎉 Emotional State Detection Test PASSED");
}