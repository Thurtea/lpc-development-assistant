#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use std::process::Command;
use std::thread;
use std::time::Duration;

use tauri::{Window, Manager, AppHandle};
use tauri::Emitter;

use lpc_dev_assistant::{OllamaClient, ContextManager, PromptBuilder, MudReferenceIndex};

mod wsl;
mod driver;

use driver::pipeline::{CompileResult, DriverPipeline};
use wsl::PathMapper;

#[derive(Clone)]
pub struct AppState {
    pub workspace_root: PathBuf,
    pub prompt_builder: Arc<PromptBuilder>,
    pub index: Arc<RwLock<MudReferenceIndex>>,
    pub cancel_flag: Arc<AtomicBool>,
    pub first_run: Arc<AtomicBool>,
    pub path_mapper: Arc<PathMapper>,
}

#[tauri::command]
fn get_setup_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let ollama_ok = match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            match OllamaClient::new() {
                Ok(client) => rt.block_on(async { client.list_models().await.is_ok() }),
                Err(_) => false,
            }
        }
        Err(_) => false,
    };

    let templates_exist = state.workspace_root.join("templates").exists();
    let corpus_exists = state.workspace_root.join("mud-references").exists();
    let index_exists = state.workspace_root.join(".index").exists();

    Ok(serde_json::json!({
        "ollama_installed": ollama_ok,
        "templates_exist": templates_exist,
        "corpus_exists": corpus_exists,
        "index_built": index_exists,
        "first_run": state.first_run.load(Ordering::SeqCst)
    }))
}

#[tauri::command]
async fn run_initial_setup(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let cm = ContextManager::new(state.workspace_root.clone());
    cm.ensure_templates_exist().map_err(|e| e.to_string())?;

    let corpus_root = state.workspace_root.join("mud-references").join("extracted");
    if !corpus_root.exists() {
        let archives_dir = state.workspace_root.join("mud-references");
        if archives_dir.exists() {
            cm.extract_archives().map_err(|e| e.to_string())?;
        }
    }

    let mut index = state.index.write().map_err(|e| e.to_string())?;
    match index.build_index() {
        Ok(count) => {
            state.first_run.store(false, Ordering::SeqCst);
            Ok(format!("Setup complete. Indexed {} files.", count))
        }
        Err(e) => Err(format!("Index build failed: {}", e)),
    }
}

#[tauri::command]
fn mark_setup_complete(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.first_run.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn ask_ollama(
    model: String,
    question: String,
    context_type: Option<String>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    num_predict: Option<i32>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let cm = ContextManager::new(state.workspace_root.clone());
    let _ = cm.ensure_templates_exist();

    let use_context = !matches!(context_type.as_deref(), Some("none"));

    // Gather examples from the index (top 5)
    let examples: Vec<String> = if use_context {
        let guard = state.index.read().map_err(|e| e.to_string())?;
        match guard.search_relevant_code(&question, 5) {
            Ok(res) => res
                .into_iter()
                .map(|snip| format!("// File: {}\n{}\n", snip.path.display(), snip.snippet))
                .collect(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let prompt = state
        .prompt_builder
        .build_prompt(&question, &model, examples)
        .map_err(|e| e.to_string())?;

    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    let options = Some(lpc_dev_assistant::ollama_client::OllamaOptions::with_defaults(
        temperature,
        top_p,
        top_k,
        num_predict,
    ));
    client
        .generate(&model, &prompt, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ask_ollama_stream(
    window: Window,
    model: String,
    question: String,
    context_type: Option<String>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    num_predict: Option<i32>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let cm = ContextManager::new(state.workspace_root.clone());
    let _ = cm.ensure_templates_exist();

    let use_context = !matches!(context_type.as_deref(), Some("none"));

    let examples: Vec<String> = if use_context {
        let guard = state.index.read().map_err(|e| e.to_string())?;
        match guard.search_relevant_code(&question, 5) {
            Ok(res) => res
                .into_iter()
                .map(|snip| format!("// File: {}\n{}\n", snip.path.display(), snip.snippet))
                .collect(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let prompt = state
        .prompt_builder
        .build_prompt(&question, &model, examples)
        .map_err(|e| e.to_string())?;

    state.cancel_flag.store(false, Ordering::SeqCst);
    let cancel_flag = state.cancel_flag.clone();

    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn(async move {
        let mut stream = client.generate_stream_with_cancel(
            &model,
            &prompt,
            Some(lpc_dev_assistant::ollama_client::OllamaOptions::with_defaults(
                temperature,
                top_p,
                top_k,
                num_predict,
            )),
            cancel_flag,
        );
        use tokio_stream::StreamExt;
        while let Some(item) = stream.next().await {
            match item {
                Ok(tok) => {
                    let _ = window.emit("ollama-token", tok);
                }
                Err(e) => {
                    let _ = window.emit("ollama-error", e.to_string());
                    break;
                }
            }
        }
        let _ = window.emit("ollama-done", ());
    });

    Ok(())
}

#[tauri::command]
fn list_models() -> Result<Vec<String>, String> {
    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async { client.list_models().await.map_err(|e| e.to_string()) })
}

#[tauri::command]
async fn check_ollama_available() -> Result<bool, String> {
    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    Ok(client.list_models().await.is_ok())
}

#[tauri::command]
async fn get_available_models() -> Result<Vec<String>, String> {
    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    client.list_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_ollama_service() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("cmd")
            .args(&["/C", "start", "", "ollama", "serve"])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        tokio::process::Command::new("sh")
            .args(&["-c", "ollama serve &"])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
async fn stop_generation(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn check_ollama_health() -> Result<serde_json::Value, String> {
    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    match rt.block_on(async { client.list_models().await.map_err(|e| e.to_string()) }) {
        Ok(models) => Ok(serde_json::json!({"ok": true, "models": models})),
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn start_ollama() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "cmd", "/K", "ollama", "serve"])
            .spawn()
            .map_err(|e| format!("Failed to start Ollama: {}", e))?;
        // Give Ollama time to spin up, then retry connectivity a few times
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        for attempt in 0..3 {
            thread::sleep(Duration::from_secs(2));
            if let Ok(client) = OllamaClient::new() {
                let ok = rt.block_on(async { client.list_models().await.is_ok() });
                if ok {
                    return Ok("Ollama started and reachable".to_string());
                }
            }
            if attempt == 2 {
                return Ok("Ollama started, but connection not confirmed. Please wait a moment then retry.".to_string());
            }
        }
        Ok("Ollama start attempted".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .args(&["-c", "ollama serve &"])
            .spawn()
            .map_err(|e| format!("Failed to start Ollama: {}", e))?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        for attempt in 0..3 {
            thread::sleep(Duration::from_secs(2));
            if let Ok(client) = OllamaClient::new() {
                let ok = rt.block_on(async { client.list_models().await.is_ok() });
                if ok {
                    return Ok("Ollama started and reachable".to_string());
                }
            }
            if attempt == 2 {
                return Ok("Ollama started, but connection not confirmed. Please wait a moment then retry.".to_string());
            }
        }
        Ok("Ollama start attempted".to_string())
    }
}

#[tauri::command]
fn analyze_driver(path: String) -> Result<serde_json::Value, String> {
    lpc_dev_assistant::driver_analyzer::efuns_json(&path)
        .and_then(|v| serde_json::to_value(&v).map_err(|e| e.to_string()))
}

#[tauri::command]
fn list_contexts(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let templates = state.workspace_root.join("templates");
    if templates.join("driver_context.txt").exists() {
        out.push("driver".to_string());
    }
    if templates.join("mudlib_context.txt").exists() {
        out.push("mudlib".to_string());
    }
    if templates.join("efuns_context.txt").exists() {
        out.push("efuns".to_string());
    }
    if templates.join("reference_sources.txt").exists() {
        out.push("references".to_string());
    }
    if out.is_empty() {
        out.push("none".to_string());
    }
    Ok(out)
}

#[tauri::command]
fn extract_references(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let cm = ContextManager::new(state.workspace_root.clone());
    cm.extract_archives().map_err(|e: anyhow::Error| e.to_string())?;
    Ok("Extraction completed".to_string())
}

#[tauri::command]
fn search_examples(keyword: String, state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let cm = ContextManager::new(state.workspace_root.clone());
    let results = cm.search_code_examples(&keyword);
    Ok(results.iter().map(|p| p.display().to_string()).collect())
}

#[tauri::command]
fn save_response(filename: String, contents: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let path = state.workspace_root.join(filename);
    std::fs::write(&path, contents).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn compile_lpc(
    file_path: String,
    state: tauri::State<'_, AppState>
) -> Result<CompileResult, String> {
    let pipeline = DriverPipeline::new(state.path_mapper.clone());

    pipeline.compile(&file_path, |_| {})
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn build_driver_ui(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let pipeline = DriverPipeline::new(state.path_mapper.clone());
    let result = pipeline
        .build_ui(|_ev| {})
        .await
        .map_err(|e| e.to_string())?;
    if result.success {
        Ok("build-ui completed".to_string())
    } else {
        Err(format!("build-ui failed (code {:?}): {:?}", result.exit_code, result.stderr))
    }
}

#[tauri::command]
async fn test_driver_connection(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let executor = crate::wsl::command_executor::WslExecutor::new(
        state.path_mapper.wsl_driver_root().to_string()
    );

    let wsl_check = tokio::process::Command::new("wsl.exe")
        .args(["--status"])
        .output()
        .await;

    if wsl_check.is_err() {
        return Err("WSL is not installed or not accessible. Please install WSL and try again.".to_string());
    }

    match executor.execute("pwd && ls -la ./build/driver 2>&1").await {
        Ok(result) => {
            let output = format!("{}\n{}", result.stdout.join("\n"), result.stderr.join("\n"));

            if result.stdout.iter().any(|line| line.contains("driver")) {
                Ok(format!("Driver found.\n\nWorking directory: {}\n\nDriver file details:\n{}",
                    state.path_mapper.wsl_driver_root(),
                    output
                ))
            } else {
                Err(format!("Driver binary not found at ./build/driver\n\nWorking directory: {}\n\nDirectory contents:\n{}",
                    state.path_mapper.wsl_driver_root(),
                    output
                ))
            }
        }
        Err(e) => Err(format!("WSL connection failed: {}", e))
    }
}

#[tauri::command]fn search_references(query: String, limit: Option<usize>, state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let guard = state.index.read().map_err(|e| e.to_string())?;
    let max_hits = limit.unwrap_or(15);
    match guard.search_with_scoring(&query, max_hits) {
        Ok(results) => {
            let json_results = results
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "path": r.path.display().to_string(),
                        "line_number": r.line_number,
                        "snippet": r.snippet,
                        "relevance_score": r.relevance_score,
                        "file_type": r.file_type
                    })
                })
                .collect();
            Ok(json_results)
        }
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    eprintln!("Starting LPC Dev Assistant...");

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    eprintln!("Working directory: {:?}", cwd);

    let first_run_flag = !cwd.join(".setup_complete").exists();

    let templates_dir = cwd.join("templates");
    eprintln!("Templates directory: {:?}", templates_dir);

    let pb = match PromptBuilder::new(templates_dir.clone()) {
        Ok(p) => {
            eprintln!("PromptBuilder initialized successfully");
            p
        }
        Err(e) => {
            eprintln!("Warning: PromptBuilder init failed: {}, using empty builder", e);
            PromptBuilder::new_empty(templates_dir)
        }
    };

    let index_dir = cwd.join(".index");
    let corpus_root = cwd.join("mud-references").join("extracted");

    eprintln!("Initializing MudReferenceIndex...");
    eprintln!("Index dir: {:?}", index_dir);
    eprintln!("Corpus root: {:?}", corpus_root);

    let mut mud_index = match MudReferenceIndex::open_or_build(&index_dir, &corpus_root) {
        Ok(idx) => {
            eprintln!("MudReferenceIndex initialized successfully");
            idx
        }
        Err(e) => {
            eprintln!("Warning: MudReferenceIndex init failed: {}, using empty index", e);
            MudReferenceIndex::new(corpus_root.clone())
        }
    };

    if corpus_root.exists() {
        match mud_index.build_index() {
            Ok(count) => eprintln!("Indexed {} files from mud-references", count),
            Err(e) => eprintln!("Failed to build index: {}", e),
        }
    } else {
        eprintln!("Skipping index build (corpus missing)");
    }

    let path_mapper = PathMapper::new(
        cwd.clone(),
        "/home/thurtea/amlp-driver".to_string(),
        "/home/thurtea/amlp-library".to_string(),
    );

    let state = AppState {
        workspace_root: cwd.clone(),
        prompt_builder: Arc::new(pb),
        index: Arc::new(RwLock::new(mud_index)),
        cancel_flag: Arc::new(AtomicBool::new(false)),
        first_run: Arc::new(AtomicBool::new(first_run_flag)),
        path_mapper: Arc::new(path_mapper),
    };

    eprintln!("Building Tauri app...");
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            ask_ollama,
            ask_ollama_stream,
            check_ollama_health,
            start_ollama,
            analyze_driver,
            list_models,
            get_available_models,
            check_ollama_available,
            start_ollama_service,
            stop_generation,
            list_contexts,
            extract_references,
            search_examples,
            search_references,
            save_response,
            get_setup_status,
            run_initial_setup,
            mark_setup_complete,
            compile_lpc,
            build_driver_ui,
            test_driver_connection
        ])
        .setup(|app| {
            let app_handle = app.handle();
            let state = app_handle.state::<AppState>();
            
            if state.first_run.load(Ordering::SeqCst) {
                eprintln!("First run detected - setup required");
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
