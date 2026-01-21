use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

mod ollama_client {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ollama_client.rs"));
}
mod context_manager {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/context_manager.rs"));
}

#[tokio::main]
async fn main() {
    println!("RUN_UI_TESTS: Starting backend UI tests");
    let client = match ollama_client::OllamaClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create OllamaClient: {}", e);
            return;
        }
    };
    let workspace_root = PathBuf::from("E:\\Work\\AMLP");
    let ctx = context_manager::ContextManager::new(workspace_root.clone());

    // Test 1: list_models
    println!("\n[TEST 1] list_models");
    match client.list_models().await {
        Ok(models) => println!("Models: {:?}", models),
        Err(e) => eprintln!("list_models error: {}", e),
    }

    // Test 2: extract_references
    println!("\n[TEST 2] extract_references");
    match ctx.extract_archives() {
        Ok(_) => println!("Extraction: success"),
        Err(e) => eprintln!("Extraction error: {}", e),
    }

    // Test 3: search_examples('call_other')
    println!("\n[TEST 3] search_examples('call_other')");
    let results = ctx.search_code_examples("call_other");
    println!("Found {} matches", results.len());
    for (i, p) in results.iter().take(20).enumerate() {
        println!("{}: {}", i+1, p.display());
    }

    // Test 4: ask_ollama (heart_beat efun)
    println!("\n[TEST 4] ask_ollama (Implement LPC heart_beat efun)");
    let driver_ctx = match ctx.load_driver_context() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load driver context: {}", e);
            String::new()
        }
    };

    let question = "Implement LPC heart_beat efun";
    let full_prompt = if !driver_ctx.is_empty() {
        format!("{}\n\n---\n\nQUESTION:\n{}\n\nProvide complete, production-ready C code with detailed comments.", driver_ctx, question)
    } else {
        question.to_string()
    };

    match client.generate("qwen2.5-coder:7b", &full_prompt, None).await {
        Ok(resp) => {
            println!("Received response (len={})", resp.len());
            // Save to file with timestamp
            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let filename = workspace_root.join(format!("heart_beat_{}.c", ts));
            match File::create(&filename) {
                Ok(mut f) => {
                    if let Err(e) = f.write_all(resp.as_bytes()) {
                        eprintln!("Failed to write response to {}: {}", filename.display(), e);
                    } else {
                        println!("Saved response to {}", filename.display());
                    }
                }
                Err(e) => eprintln!("Failed to create file {}: {}", filename.display(), e),
            }
        }
        Err(e) => eprintln!("ask_ollama error: {}", e),
    }

    println!("\nRUN_UI_TESTS COMPLETE");
}
