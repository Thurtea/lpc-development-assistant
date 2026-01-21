use std::fs;
use std::path::PathBuf;
use lpc_dev_assistant::{MudReferenceIndex, PromptBuilder};

#[test]
fn test_mud_reference_index_creation() {
    let temp_dir = std::env::temp_dir().join("rag_test_index");
    let corpus_dir = std::env::temp_dir().join("rag_test_corpus");
    
    // Clean up if exists
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_dir_all(&corpus_dir);
    
    // Create corpus directory
    fs::create_dir_all(&corpus_dir).unwrap();
    
    // Create index
    let index = MudReferenceIndex::open_or_build(&temp_dir, &corpus_dir);
    assert!(index.is_ok(), "Failed to create MudReferenceIndex");
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_dir_all(&corpus_dir);
}

#[test]
fn test_prompt_builder_creation() {
    let temp_dir = std::env::temp_dir().join("rag_test_templates");
    
    // Clean up if exists
    let _ = fs::remove_dir_all(&temp_dir);
    
    // Create templates directory
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a simple template file
    fs::write(
        temp_dir.join("driver_context.txt"),
        "This is a driver context template.\n"
    ).unwrap();
    
    // Create PromptBuilder
    let pb = PromptBuilder::new(temp_dir.clone());
    assert!(pb.is_ok(), "Failed to create PromptBuilder");
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_prompt_builder_empty() {
    let temp_dir = PathBuf::from("/tmp/nonexistent_path");
    let pb = PromptBuilder::new_empty(temp_dir);
    
    // Should always succeed
    let prompt = pb.build_prompt("test query", "model", vec![]);
    assert!(prompt.is_ok(), "Failed to build empty prompt");
    
    let prompt_text = prompt.unwrap();
    assert!(prompt_text.contains("LPC MUD"), "Prompt should contain system context");
    assert!(prompt_text.contains("test query"), "Prompt should contain user query");
}

#[test]
fn test_search_functionality() {
    let temp_dir = std::env::temp_dir().join("rag_test_search");
    let corpus_dir = std::env::temp_dir().join("rag_test_search_corpus");
    
    // Clean up if exists
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_dir_all(&corpus_dir);
    
    // Create corpus directory and test files
    fs::create_dir_all(&corpus_dir).unwrap();
    
    // Create a test reference file
    fs::write(
        corpus_dir.join("test.txt"),
        "This is a test file about heart_beat and master objects.\n\
         The heart_beat is a special function in LPC.\n\
         Master objects control the mudlib.\n"
    ).unwrap();
    
    // Create index and build it
    let mut index = MudReferenceIndex::open_or_build(&temp_dir, &corpus_dir)
        .expect("Failed to create index");
    
    let indexed = index.build_index().expect("Failed to build index");
    println!("Indexed {} files", indexed);
    assert!(indexed > 0, "Should have indexed at least one file");
    
    // Test substring search
    let results = index.search_relevant_code("heart_beat", 5)
        .expect("Search failed");
    
    println!("Substring search results: {} found", results.len());
    assert!(!results.is_empty(), "Should find search results");
    
    // Test scoring search
    let scored_results = index.search_with_scoring("heart_beat function", 5)
        .expect("Scored search failed");
    
    println!("Scored search results: {} found", scored_results.len());
    assert!(!scored_results.is_empty(), "Should find scored results");
    
    if !scored_results.is_empty() {
        let first = &scored_results[0];
        println!("Top result: {}", first.path.display());
        println!("Relevance score: {:.2}%", first.relevance_score * 100.0);
        assert!(first.relevance_score > 0.0, "Relevance score should be positive");
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_dir_all(&corpus_dir);
}

#[test]
fn test_prompt_building_with_examples() {
    let temp_dir = PathBuf::from("/tmp");
    let pb = PromptBuilder::new_empty(temp_dir);
    
    let examples = vec![
        "// Example 1\nvoid heart_beat() { }".to_string(),
        "// Example 2\nvoid setup() { }".to_string(),
    ];
    
    let prompt = pb.build_prompt("How do I implement heart_beat?", "llama2", examples)
        .expect("Failed to build prompt");
    
    assert!(prompt.contains("Example 1"), "Prompt should contain examples");
    assert!(prompt.contains("heart_beat"), "Prompt should contain user query");
}

#[test]
fn test_token_estimation() {
    let text = "This is a test string for token estimation.";
    let tokens = PromptBuilder::estimate_tokens(text);
    
    println!("Estimated tokens for '{}': {}", text, tokens);
    
    // 44 chars / 4 + 1 = 12 tokens
    assert!(tokens > 0, "Token count should be positive");
    assert!(tokens < 50, "Token count should be reasonable");
}

#[test]
fn test_prompt_trimming() {
    let temp_dir = PathBuf::from("/tmp");
    let pb = PromptBuilder::new_empty(temp_dir);
    
    let parts = vec!["Hello", "world", "this is a test"];
    let result = pb.trim_to_fit(parts, 100);
    
    assert!(result.contains("Hello"), "Result should contain first part");
    assert!(result.contains("world"), "Result should contain second part");
}
