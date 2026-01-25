use anyhow::{Result, Context};
use lpc_dev_assistant::{OllamaClient, PromptBuilder, MudReferenceIndex, RAGValidator, OllamaOptions};
use std::path::PathBuf;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    println!("LPC Development Assistant - Enhanced Query CLI");
    println!("================================================\n");

    // Configuration
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mud_references_path = workspace_root.join("mud-references");
    let templates_path = workspace_root.join("templates");
    
    // Initialize components
    println!("Initializing...");
    let ollama_client = OllamaClient::new()
        .context("Failed to create Ollama client. Is Ollama running?")?;
    
    let mut mud_index = MudReferenceIndex::new(mud_references_path.clone());
    println!("Building index (this may take a moment)...");
    let indexed_files = mud_index.build_index()?;
    println!("  ✓ Indexed {} files", indexed_files);
    
    let validator = RAGValidator::new(mud_index.clone(), mud_references_path.clone())?;
    println!("  ✓ Loaded {} known efuns", validator.get_known_efuns().len());
    
    let prompt_builder = PromptBuilder::new(templates_path)
        .unwrap_or_else(|_| PromptBuilder::new_empty(workspace_root.join("templates")))
        .with_reference_index(mud_index);
    
    // Get available models
    let models = ollama_client.list_models().await?;
    if models.is_empty() {
        eprintln!("No models available. Please pull a model first:");
        eprintln!("  ollama pull deepseek-coder");
        return Ok(());
    }
    
    // Select model
    println!("\nAvailable models:");
    for (i, model) in models.iter().enumerate() {
        println!("  {}. {}", i + 1, model);
    }
    
    print!("\nSelect model (1-{}, default 1): ", models.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let model_idx = input.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
    let model_name = models.get(model_idx).unwrap_or(&models[0]);
    
    println!("\nUsing model: {}\n", model_name);
    println!("Type your LPC questions (or 'quit' to exit)");
    println!("Examples:");
    println!("  - How do I use call_other()?");
    println!("  - Show me a combat system example");
    println!("  - What's the difference between mappings and arrays?");
    println!();
    
    // Query loop
    loop {
        print!("/> ");
        io::stdout().flush()?;
        
        let mut query = String::new();
        io::stdin().read_line(&mut query)?;
        let query = query.trim();
        
        if query.is_empty() {
            continue;
        }
        
        if query.eq_ignore_ascii_case("quit") || query.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }
        
        // Validate and retrieve sources
        println!("\n🔍 Searching knowledge base...");
        let validation = validator.validate_query(query)?;
        
        // Display validation results
        println!("\n📚 Source Validation:");
        println!("  Confidence Score: {:.1}%", validation.confidence_score * 100.0);
        println!("  Validation Score: {:.1}%", validation.validation_score * 100.0);
        println!("  Sources Consulted: {}", validation.sources_consulted);
        
        if !validation.efuns_found.is_empty() {
            println!("  Efuns Found: {}", validation.efuns_found.join(", "));
        }
        
        // Show top sources
        if !validation.retrieved_documents.is_empty() {
            println!("\n📄 Top Sources:");
            for (i, doc) in validation.retrieved_documents.iter().take(5).enumerate() {
                let status_icon = match doc.validation_status {
                    lpc_dev_assistant::rag_validator::ValidationStatus::Verified => "✓",
                    lpc_dev_assistant::rag_validator::ValidationStatus::CrossReferenced => "⊕",
                    lpc_dev_assistant::rag_validator::ValidationStatus::SingleSource => "○",
                    lpc_dev_assistant::rag_validator::ValidationStatus::Uncertain => "?",
                };
                
                // Extract filename from path
                let filename = std::path::Path::new(&doc.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&doc.path);
                
                println!("  {} [{}] {} (line {}) - {:.0}%",
                    i + 1,
                    status_icon,
                    filename,
                    doc.line_number,
                    doc.relevance_score * 100.0
                );
            }
        }
        
        // Show code examples if available
        if !validation.code_examples.is_empty() {
            println!("\n💡 Code Examples Found:");
            for (i, example) in validation.code_examples.iter().take(3).enumerate() {
                println!("  {}. From {} driver", i + 1, example.driver);
            }
        }
        
        // Build prompt with RAG context
        println!("\n🤖 Generating response with {} model...", model_name);
        let prompt = prompt_builder.build_prompt(query, model_name, Vec::new())
            .map_err(|e| anyhow::anyhow!("Failed to build prompt: {}", e))?;
        
        // Generate response
        let options = OllamaOptions::with_defaults(
            Some(0.3),  // Low temperature for more consistent answers
            Some(0.9),
            Some(40),
            Some(4096),
        );
        
        let response = ollama_client.generate(model_name, &prompt, Some(options)).await?;
        
        // Display response
        println!("\n📝 Answer:");
        println!("{}", "=".repeat(70));
        println!("{}", response);
        println!("{}", "=".repeat(70));
        println!();
        
        // Show citation footer
        if validation.sources_consulted > 0 {
            println!("Answer based on {} source{} from the MUD reference corpus",
                validation.sources_consulted,
                if validation.sources_consulted == 1 { "" } else { "s" }
            );
        }
        
        println!();
    }
    
    Ok(())
}
