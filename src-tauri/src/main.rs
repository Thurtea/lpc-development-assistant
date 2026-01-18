use std::path::PathBuf;
use std::sync::Arc;

use tauri::{Manager, Window};
use tauri::Emitter;

use lpc_dev_assistant::{OllamaClient, ContextManager, PromptBuilder, MudReferenceIndex};
use std::sync::RwLock;

mod tauri_adapters {
    // Thin adapters calling into the library crate (lpc-dev-assistant src/)
    // Keep error types simple (String)
    use super::{OllamaClient, ContextManager};

    pub async fn ask_ollama(model: String, question: String, context_type: Option<String>, state: &crate::AppState) -> Result<String, String> {
        let cm = ContextManager::new(state.workspace_root.clone());
        cm.ensure_templates_exist().map_err(|e: anyhow::Error| e.to_string())?;

        // Build prompt from templates (simple concat for now)
        let mut prompt = String::new();
        if let Some(ctx) = context_type.as_deref() {
            match ctx {
                "driver" => prompt.push_str(&cm.load_driver_context().map_err(|e: anyhow::Error| e.to_string())?),
                "mudlib" => prompt.push_str(&cm.load_mudlib_context().map_err(|e: anyhow::Error| e.to_string())?),
                "efuns" => prompt.push_str(&cm.load_efuns_context().map_err(|e: anyhow::Error| e.to_string())?),
                "references" => prompt.push_str(&cm.load_reference_sources_context().map_err(|e: anyhow::Error| e.to_string())?),
                _ => {}
            }
        }
        prompt.push_str("\n\nQUESTION:\n");
        prompt.push_str(&question);

        let client = OllamaClient::new();
        client.generate(&model, &prompt).await.map_err(|e: anyhow::Error| e.to_string())
    }

    pub fn extract_references(state: &crate::AppState) -> Result<String, String> {
        let cm = ContextManager::new(state.workspace_root.clone());
        cm.extract_archives().map_err(|e: anyhow::Error| e.to_string())?;
        Ok("Extraction completed".to_string())
    }

    pub fn search_examples(state: &crate::AppState, keyword: String) -> Result<Vec<String>, String> {
        let cm = ContextManager::new(state.workspace_root.clone());
        let results = cm.search_code_examples(&keyword);
        Ok(results.iter().map(|p| p.display().to_string()).collect())
    }

    pub fn save_response(filename: String, contents: String, state: &crate::AppState) -> Result<(), String> {
        let path = state.workspace_root.join(filename);
        std::fs::write(&path, contents).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppState {
    pub workspace_root: PathBuf,
    pub prompt_builder: Arc<PromptBuilder>,
    pub index: Arc<RwLock<MudReferenceIndex>>,
    // future: config, ollama client instance
}

#[tauri::command]
async fn ask_ollama(model: String, question: String, context_type: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    // ensure templates exist (so PromptBuilder can read them)
    let cm = ContextManager::new(state.workspace_root.clone());
    let _ = cm.ensure_templates_exist();

    // Gather examples from the index (top 5)
    let examples: Vec<String> = {
        let guard = state.index.read().map_err(|e| e.to_string())?;
        match guard.search_relevant_code(&question, 5) {
            Ok(res) => res.into_iter().map(|snip| format!("// File: {}\n{}\n", snip.path.display(), snip.snippet)).collect(),
            Err(_) => Vec::new(),
        }
    };

    let prompt = state.prompt_builder.build_prompt(&question, &model, examples)
        .map_err(|e| e.to_string())?;

    // Debug output for prompt verification
    let prompt_len = prompt.len();
    let first_n = if prompt_len > 500 { 500 } else { prompt_len };
    eprintln!("DEBUG: Final prompt length: {} chars", prompt_len);
    eprintln!("DEBUG: First {} chars:\n{}", first_n, &prompt[..first_n]);

    let client = OllamaClient::new();
    client.generate(&model, &prompt).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn list_models() -> Result<Vec<String>, String> {
    let client = OllamaClient::new();
    // run in blocking context (reqwest is async but list_models is async) — use a small runtime
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async { client.list_models().await.map_err(|e| e.to_string()) })
}

#[tauri::command]
fn check_ollama_health() -> Result<serde_json::Value, String> {
    // Returns { ok: bool, models: [...] } on success, or Err(string) on failure
    let client = OllamaClient::new();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    match rt.block_on(async { client.list_models().await.map_err(|e| e.to_string()) }) {
        Ok(models) => Ok(serde_json::json!({"ok": true, "models": models})),
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn analyze_driver(path: String) -> Result<serde_json::Value, String> {
    // Analyze driver sources under given path and return JSON value.
    // `efuns_json` returns Result<Vec<(String,String)>, String>, convert that Vec into serde_json::Value.
    lpc_dev_assistant::driver_analyzer::efuns_json(&path)
        .and_then(|v| serde_json::to_value(&v).map_err(|e| e.to_string()))
}

#[tauri::command]
fn list_contexts(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let templates = state.workspace_root.join("templates");
    if templates.join("driver_context.txt").exists() { out.push("driver".to_string()); }
    if templates.join("mudlib_context.txt").exists() { out.push("mudlib".to_string()); }
    if templates.join("efuns_context.txt").exists() { out.push("efuns".to_string()); }
    if templates.join("reference_sources.txt").exists() { out.push("references".to_string()); }
    if out.is_empty() { out.push("none".to_string()); }
    Ok(out)
}

#[tauri::command]
fn extract_references(state: tauri::State<'_, AppState>) -> Result<String, String> {
    tauri_adapters::extract_references(&state)
}

#[tauri::command]
fn search_examples(keyword: String, state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    tauri_adapters::search_examples(&state, keyword)
}

#[tauri::command]
fn save_response(filename: String, contents: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    tauri_adapters::save_response(filename, contents, &state)
}

#[tauri::command]
fn ask_ollama_stream(window: Window, model: String, question: String, context_type: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Ensure templates exist and build prompt via PromptBuilder
    let cm = ContextManager::new(state.workspace_root.clone());
    let _ = cm.ensure_templates_exist();

    let examples: Vec<String> = {
        let guard = state.index.read().map_err(|e| e.to_string())?;
        match guard.search_relevant_code(&question, 5) {
            Ok(res) => res.into_iter().map(|snip| format!("// File: {}\n{}\n", snip.path.display(), snip.snippet)).collect(),
            Err(_) => Vec::new(),
        }
    };

    let prompt = state.prompt_builder.build_prompt(&question, &model, examples)
        .map_err(|e| e.to_string())?;

    // Debug output for prompt verification
    let prompt_len = prompt.len();
    let first_n = if prompt_len > 500 { 500 } else { prompt_len };
    eprintln!("DEBUG: Final prompt length: {} chars", prompt_len);
    eprintln!("DEBUG: First {} chars:\n{}", first_n, &prompt[..first_n]);

    let client = OllamaClient::new();
    let mut stream = client.generate_stream(&model, &prompt);

    // spawn async task to forward tokens to the webview
    tauri::async_runtime::spawn(async move {
        use tokio_stream::StreamExt;
        while let Some(item) = stream.next().await {
            match item {
                Ok(tok) => { let _ = window.emit("ollama-token", tok); }
                Err(e) => { let _ = window.emit("ollama-error", e.to_string()); break; }
            }
        }
        let _ = window.emit("ollama-done", ());
    });

    Ok(())
}

fn main() {
    eprintln!("🚀 Starting LPC Dev Assistant...");
    
    // workspace root is parent of this binary's current dir
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    eprintln!("📁 Working directory: {:?}", cwd);

    // Build PromptBuilder using templates folder under the current crate
    let templates_dir = cwd.join("templates");
    eprintln!("📋 Templates directory: {:?}", templates_dir);
    
    let pb = match PromptBuilder::new(templates_dir.clone()) {
        Ok(p) => {
            eprintln!("✅ PromptBuilder initialized");
            p
        }
        Err(e) => {
            eprintln!("⚠️  PromptBuilder init failed: {}, using empty builder", e);
            PromptBuilder::new_empty(templates_dir)
        }
    };

    // Initialize index at workspace_root/.index using corpus at workspace_root/mud-references/extracted
    let index_dir = cwd.join(".index");
    let corpus_root = cwd.join("mud-references").join("extracted");
    
    eprintln!("🔍 Initializing MudReferenceIndex...");
    eprintln!("   Index dir: {:?}", index_dir);
    eprintln!("   Corpus root: {:?}", corpus_root);
    
    let mud_index = match MudReferenceIndex::open_or_build(&index_dir, &corpus_root) {
        Ok(idx) => {
            eprintln!("✅ MudReferenceIndex initialized");
            idx
        }
        Err(e) => {
            eprintln!("⚠️  MudReferenceIndex init failed: {}, using empty index", e);
            // Create a minimal empty index instead of panicking
            MudReferenceIndex::new(corpus_root)
        }
    };

    let state = AppState { 
        workspace_root: cwd, 
        prompt_builder: std::sync::Arc::new(pb), 
        index: std::sync::Arc::new(RwLock::new(mud_index)) 
    };

    eprintln!("🔧 Building Tauri app...");
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            ask_ollama,
            ask_ollama_stream,
            check_ollama_health,
            analyze_driver,
            list_models,
            list_contexts,
            extract_references,
            search_examples,
            save_response
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
