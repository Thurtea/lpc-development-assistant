mod ollama_client {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ollama_client.rs"));
}

#[tokio::main]
async fn main() {
    let base = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    println!("OLLAMA HEALTH CHECK: probing {}", base);

    let client = ollama_client::OllamaClient::new();

    match client.list_models().await {
        Ok(models) => {
            println!("Ollama reachable. {} models/tags:", models.len());
            for m in models {
                println!("- {}", m);
            }
        }
        Err(e) => {
            eprintln!("Ollama unreachable: {}", e);
            std::process::exit(2);
        }
    }
}
