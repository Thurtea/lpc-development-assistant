use anyhow::{Result, Context};
use lpc_dev_assistant::{OllamaClient, PromptBuilder, MudReferenceIndex, RAGValidator, ModelTester};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("LPC Development Assistant - Model Comparison Tool");
    println!("===================================================\n");

    // Configuration
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mud_references_path = workspace_root.join("mud-references");
    let templates_path = workspace_root.join("templates");
    
    // Check if Ollama is running
    println!("Checking Ollama connection...");
    let ollama_client = OllamaClient::new()
        .context("Failed to create Ollama client. Is Ollama running?")?;
    
    // List available models
    println!("Fetching available models...");
    let available_models = ollama_client.list_models().await
        .context("Failed to list models. Ensure Ollama is running.")?;
    
    if available_models.is_empty() {
        eprintln!("No models found. Please pull some models first:");
        eprintln!("  ollama pull deepseek-coder");
        eprintln!("  ollama pull codellama");
        eprintln!("  ollama pull llama3");
        eprintln!("  ollama pull mistral");
        return Ok(());
    }
    
    println!("Available models: {:?}\n", available_models);
    
    // Select models to test (from candidates list)
    let test_candidates = vec!["deepseek-coder", "codellama", "llama3", "mistral"];
    let mut models_to_test = Vec::new();
    
    for candidate in &test_candidates {
        if available_models.iter().any(|m| m.starts_with(candidate)) {
            // Find the exact model name
            if let Some(model) = available_models.iter().find(|m| m.starts_with(candidate)) {
                models_to_test.push(model.clone());
            }
        }
    }
    
    if models_to_test.is_empty() {
        eprintln!("None of the recommended models are available.");
        eprintln!("Available: {:?}", available_models);
        eprintln!("Recommended: {:?}", test_candidates);
        
        // Ask user if they want to test with available models
        println!("\nWould you like to test with the first 4 available models? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            models_to_test = available_models.iter().take(4).cloned().collect();
        } else {
            return Ok(());
        }
    }
    
    println!("Testing models: {:?}\n", models_to_test);
    
    // Initialize RAG components
    println!("Loading MUD reference index...");
    let mut mud_index = MudReferenceIndex::new(mud_references_path.clone());
    let indexed_files = mud_index.build_index()
        .context("Failed to build MUD reference index")?;
    println!("  Indexed {} files\n", indexed_files);
    
    println!("Initializing RAG validator...");
    let validator = RAGValidator::new(mud_index.clone(), mud_references_path.clone())
        .context("Failed to create RAG validator")?;
    println!("  Loaded {} known efuns\n", validator.get_known_efuns().len());
    
    println!("Loading prompt templates...");
    let prompt_builder = PromptBuilder::new(templates_path)
        .unwrap_or_else(|_| PromptBuilder::new_empty(workspace_root.join("templates")))
        .with_reference_index(mud_index);
    println!("  Templates loaded\n");
    
    // Create model tester
    let tester = ModelTester::new(ollama_client, prompt_builder, Some(validator));
    
    // Get test queries
    let test_queries = ModelTester::get_default_test_queries();
    println!("Running {} test queries across {} models...\n", test_queries.len(), models_to_test.len());
    
    // Run comparison
    let comparison = tester.compare_models(models_to_test, test_queries).await
        .context("Model comparison failed")?;
    
    // Print results
    println!("\n");
    println!("=== COMPARISON RESULTS ===");
    println!();
    println!("Best Accuracy:  {}", comparison.summary.best_accuracy);
    println!("Best Speed:     {}", comparison.summary.best_speed);
    println!("Best Quality:   {}", comparison.summary.best_quality);
    println!();
    println!("RECOMMENDED MODEL: {}", comparison.summary.recommended_model);
    println!();
    println!("{}", comparison.summary.reasoning);
    println!();
    
    // Detailed results
    println!("\n=== DETAILED RESULTS ===\n");
    
    for model in &comparison.models_tested {
        let model_results: Vec<_> = comparison.results.iter()
            .filter(|r| r.model_name == *model)
            .collect();
        
        if model_results.is_empty() {
            continue;
        }
        
        println!("Model: {}", model);
        println!("{}", "=".repeat(60));
        
        let avg_accuracy: f32 = model_results.iter()
            .map(|r| r.accuracy_score)
            .sum::<f32>() / model_results.len() as f32;
        
        let avg_quality: f32 = model_results.iter()
            .map(|r| r.quality_score)
            .sum::<f32>() / model_results.len() as f32;
        
        let avg_time: f32 = model_results.iter()
            .map(|r| r.response_time_ms as f32)
            .sum::<f32>() / model_results.len() as f32;
        
        let avg_tps: f32 = model_results.iter()
            .filter_map(|r| r.tokens_per_second)
            .sum::<f32>() / model_results.len() as f32;
        
        println!("  Average Accuracy:  {:.1}%", avg_accuracy * 100.0);
        println!("  Average Quality:   {:.1}%", avg_quality * 100.0);
        println!("  Average Time:      {:.0}ms", avg_time);
        println!("  Tokens/sec:        {:.1}", avg_tps);
        println!();
        
        for result in &model_results {
            println!("  Query: {}", result.query);
            println!("    Accuracy: {:.1}% | Quality: {:.1}% | Time: {}ms",
                result.accuracy_score * 100.0,
                result.quality_score * 100.0,
                result.response_time_ms
            );
            
            if let Some(ref val) = result.validation_result {
                println!("    Sources: {} | Efuns found: {} | Confidence: {:.1}%",
                    val.sources_consulted,
                    val.efuns_found.len(),
                    val.confidence_score * 100.0
                );
            }
            println!();
        }
        println!();
    }
    
    // Save results to file
    let output_file = "model_comparison_results.json";
    tester.save_results(&comparison, output_file)?;
    
    println!("\n✓ Comparison complete! Results saved to {}", output_file);
    
    Ok(())
}
