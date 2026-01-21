use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

mod ollama_client {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ollama_client.rs"));
}
mod context_manager {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/context_manager.rs"));
}

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    println!("RUN_STREAM_TEST: Starting headless streaming test");
    let client = match ollama_client::OllamaClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create OllamaClient: {}", e);
            return;
        }
    };
    let workspace_root = PathBuf::from("E:\\Work\\AMLP");
    let ctx = context_manager::ContextManager::new(workspace_root.clone());

    let driver_ctx = match ctx.load_driver_context() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load driver context: {}", e);
            String::new()
        }
    };

    let question = "LPC mudlib inheritance example with full mudlib structure";
    let full_prompt = if !driver_ctx.is_empty() {
        format!("{}\n\n---\n\nQUESTION:\n{}\n\nProvide complete, production-ready C code with detailed comments.", driver_ctx, question)
    } else {
        question.to_string()
    };

    println!("Using model: qwen2.5-coder:7b");
    let cancel_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut stream = client.generate_stream_with_cancel("qwen2.5-coder:7b", &full_prompt, None, cancel_flag);

    use std::time::Instant;
    let start = Instant::now();
    let mut count: usize = 0;
    let mut buffer = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(tok) => {
                count += 1;
                print!("{}", tok);
                let _ = std::io::stdout().flush();
                buffer.push_str(&tok);
            }
            Err(e) => {
                eprintln!("stream error: {}", e);
            }
        }
    }

    let dur = start.elapsed().as_secs_f64();
    println!("\nSTREAM COMPLETE: tokens={} duration_s={:.2}", count, dur);

    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = workspace_root.join(format!("mudlib_inheritance_{}.c", ts));
    match File::create(&filename) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(buffer.as_bytes()) {
                eprintln!("Failed to write response to {}: {}", filename.display(), e);
            } else {
                println!("Saved response to {}", filename.display());
            }
        }
        Err(e) => eprintln!("Failed to create file {}: {}", filename.display(), e),
    }

    println!("RUN_STREAM_TEST COMPLETE");
}
