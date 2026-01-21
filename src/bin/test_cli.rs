use std::path::PathBuf;

mod ollama_client {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ollama_client.rs"));
}
mod context_manager {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/context_manager.rs"));
}

#[tokio::main]
async fn main() {
    println!("TEST RUN: Ollama + Context Manager backend tests");

    // Setup
    let client = match ollama_client::OllamaClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create OllamaClient: {}", e);
            return;
        }
    };
    let workspace_root = PathBuf::from("E:\\Work\\AMLP");
    let ctx = context_manager::ContextManager::new(workspace_root.clone());

    // TEST 1: list models
    println!("\n=== TEST 1: list_models ===");
    match client.list_models().await {
        Ok(models) => {
            println!("Models: {:?}", models);
        }
        Err(e) => {
            eprintln!("list_models error: {}", e);
        }
    }

    // TEST 2: extract references
    println!("\n=== TEST 2: extract_references ===");
    match ctx.extract_archives() {
        Ok(_) => println!("Extraction: success"),
        Err(e) => eprintln!("Extraction error: {}", e),
    }

    // TEST 3: search_examples for 'socket_efun'
    println!("\n=== TEST 3: search_examples('socket_efun') ===");
    let results = ctx.search_code_examples("socket_efun");
    println!("Found {} matches", results.len());
    for (i, p) in results.iter().take(10).enumerate() {
        println!("{}: {}", i+1, p.display());
    }

    // TEST 4: ask_ollama
    println!("\n=== TEST 4: ask_ollama ===");
    let driver_ctx = match ctx.load_driver_context() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load driver context: {}", e);
            String::new()
        }
    };

    let question = "Write LPC lexer tokenization code";
    let full_prompt = if !driver_ctx.is_empty() {
        format!("{}\n\n---\n\nQUESTION:\n{}\n\nProvide complete, production-ready C code with detailed comments.", driver_ctx, question)
    } else {
        question.to_string()
    };

    match client.generate("qwen2.5-coder:7b", &full_prompt, None).await {
        Ok(resp) => {
            let snippet: String = resp.chars().take(2000).collect();
            println!("Received response (len={}):\n{}", resp.len(), snippet);
        }
        Err(e) => eprintln!("ask_ollama error: {}", e),
    }

    println!("\nTEST RUN COMPLETE");
}
