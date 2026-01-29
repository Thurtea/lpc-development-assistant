#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use std::process::Command;
use std::thread;
use std::time::Duration;
use serde::Serialize;
// unused imports removed to satisfy clippy

use tauri::{Window, Manager};
use tauri::Emitter;

use lpc_dev_assistant::{OllamaClient, ContextManager, PromptBuilder, MudReferenceIndex};

mod wsl;
mod driver;
mod config;
mod commands;

use driver::pipeline::{CompileResult, RunResult, DriverPipeline};
use wsl::PathMapper;
use crate::config::{DriverConfig, load_driver_config, save_driver_config};
use commands::file_operations::{save_to_driver, save_to_library};

#[derive(Debug, Serialize)]
struct ComponentDiagnostic {
    name: String,
    ok: bool,
    message: String,
    fix_command: Option<String>,
}

#[derive(Debug, Serialize)]
struct WslDiagnostics {
    wsl_available: bool,
    components: Vec<ComponentDiagnostic>,
}

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
#[allow(clippy::too_many_arguments)] // Tauri command parameters - cannot be easily refactored
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
#[allow(clippy::too_many_arguments)] // Tauri command parameters - cannot be easily refactored
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
    match client.list_models().await {
        Ok(models) => {
            // Check if required model is available
            let required_model = "qwen2.5-coder:3b";
            Ok(models.iter().any(|m| m.contains(required_model)))
        }
        Err(_) => Ok(false)
    }
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
            .args(["/C", "start", "", "ollama", "serve"])
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
        Ok(models) => {
            let required_model = "qwen2.5-coder:3b";
            let has_required = models.iter().any(|m| m.contains(required_model));
            Ok(serde_json::json!({
                "ok": true,
                "models": models,
                "has_required_model": has_required,
                "required_model": required_model
            }))
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn start_ollama() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", "cmd", "/K", "ollama", "serve"])
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
fn browse_wsl_directory(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    // Use rfd to present a folder picker and attempt to map selected folder to a WSL path
    match rfd::FileDialog::new().pick_folder() {
        Some(path) => {
            if let Some(mapped) = state.path_mapper.to_wsl_driver(&path) {
                return Ok(Some(mapped));
            }
            if let Some(mapped) = state.path_mapper.to_wsl_library(&path) {
                return Ok(Some(mapped));
            }
            // If the selected path already looks like a WSL (starts with /), return as-is
            if path.starts_with(std::path::Path::new("/")) {
                return Ok(Some(path.display().to_string()));
            }
            Ok(Some(path.display().to_string()))
        }
        None => Ok(None),
    }
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
async fn run_lpc(
    file_path: String,
    state: tauri::State<'_, AppState>
) -> Result<RunResult, String> {
    if file_path.trim().is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    
    let pipeline = DriverPipeline::new(state.path_mapper.clone());

    pipeline.run(&file_path, |_| {})
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

#[tauri::command]
async fn get_driver_config() -> Result<DriverConfig, String> {
    load_driver_config()
}

#[tauri::command]
async fn save_driver_config_cmd(cfg: DriverConfig) -> Result<(), String> {
    save_driver_config(&cfg)
}

async fn run_wsl_bool(cmd: &str) -> Result<bool, String> {
    let output = tokio::process::Command::new("wsl.exe")
        .args(["-e", "bash", "-lc", cmd])
        .output()
        .await
        .map_err(|e| e.to_string())?;
    Ok(output.status.success())
}

#[tauri::command]
async fn validate_and_diagnose_wsl(config: DriverConfig) -> Result<WslDiagnostics, String> {
    let mut components = Vec::new();

    let wsl_status = tokio::process::Command::new("wsl.exe")
        .args(["--status"])
        .output()
        .await;

    let wsl_available = wsl_status.as_ref().map(|o| o.status.success()).unwrap_or(false);

    if !wsl_available {
        components.push(ComponentDiagnostic {
            name: "WSL availability".to_string(),
            ok: false,
            message: "WSL is not installed or not accessible. Install WSL and reboot.".to_string(),
            fix_command: Some("wsl --install".to_string()),
        });
        return Ok(WslDiagnostics { wsl_available, components });
    }

    // Driver directory
    let driver_dir_cmd = format!("test -d {}", config.wsl_driver_root);
    let driver_dir_ok = run_wsl_bool(&driver_dir_cmd).await.unwrap_or(false);
    components.push(ComponentDiagnostic {
        name: "Driver directory".to_string(),
        ok: driver_dir_ok,
        message: if driver_dir_ok { "Driver directory found".to_string() } else { format!("Driver directory not found at {}", config.wsl_driver_root) },
        fix_command: if driver_dir_ok { None } else { Some(format!("mkdir -p {}", config.wsl_driver_root)) },
    });

    // Driver binary
    let driver_bin = format!("{}/build/driver", config.wsl_driver_root.trim_end_matches('/'));
    let driver_bin_cmd = format!("test -x {}", driver_bin);
    let driver_bin_ok = run_wsl_bool(&driver_bin_cmd).await.unwrap_or(false);
    components.push(ComponentDiagnostic {
        name: "Driver binary".to_string(),
        ok: driver_bin_ok,
        message: if driver_bin_ok { "Driver binary is present and executable".to_string() } else { format!("Driver binary missing or not executable at {}", driver_bin) },
        fix_command: if driver_bin_ok { None } else { Some(format!("cd {} && make || cargo build --release", config.wsl_driver_root)) },
    });

    // Library directory
    let lib_dir_cmd = format!("test -d {}", config.wsl_library_root);
    let lib_dir_ok = run_wsl_bool(&lib_dir_cmd).await.unwrap_or(false);
    components.push(ComponentDiagnostic {
        name: "Library directory".to_string(),
        ok: lib_dir_ok,
        message: if lib_dir_ok { "Library directory found".to_string() } else { format!("Library directory not found at {}", config.wsl_library_root) },
        fix_command: if lib_dir_ok { None } else { Some(format!("mkdir -p {}", config.wsl_library_root)) },
    });

    Ok(WslDiagnostics { wsl_available, components })
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

// New Ollama Commands

#[tauri::command]
async fn check_ollama_installed() -> Result<bool, String> {
    let output = tokio::process::Command::new("ollama")
        .arg("--version")
        .output()
        .await;
    
    Ok(output.map(|o| o.status.success()).unwrap_or(false))
}

#[tauri::command]
async fn check_ollama_running() -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn start_ollama_server() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("cmd")
            .args(["/C", "start", "", "ollama", "serve"])
            .spawn()
            .map_err(|e| format!("Failed to start Ollama: {}", e))?;
        
        Ok("Ollama server starting...".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        tokio::process::Command::new("sh")
            .args(&["-c", "ollama serve &"])
            .spawn()
            .map_err(|e| format!("Failed to start Ollama: {}", e))?;
        
        Ok("Ollama server starting...".to_string())
    }
}

#[tauri::command]
async fn pull_ollama_model(model: String) -> Result<String, String> {
    let output = tokio::process::Command::new("ollama")
        .arg("pull")
        .arg(&model)
        .output()
        .await
        .map_err(|e| format!("Failed to pull model: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Successfully pulled model: {}", model))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to pull model: {}", stderr))
    }
}

#[tauri::command]
async fn list_ollama_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let models = json["models"]
        .as_array()
        .ok_or("Invalid response format")?
        .iter()
        .filter_map(|m| m["name"].as_str())
        .map(|s| s.to_string())
        .collect();
    
    Ok(models)
}

#[tauri::command]
async fn setup_staging_directory() -> Result<String, String> {
    // Determine staging directory path
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let staging_dir = home_dir.join(".lpc-dev-assistant").join("staging");
    
    // Create staging directory
    std::fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("Failed to create staging directory: {}", e))?;
    
    // Load config, update it, and save
    let mut cfg = load_driver_config()?;
    cfg.staging_directory = Some(staging_dir.to_string_lossy().to_string());
    cfg.setup_complete = true;
    save_driver_config(&cfg)?;
    
    Ok(format!("Staging directory created: {}", staging_dir.display()))
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

    // Load driver configuration (WSL paths) and initialize PathMapper
    let driver_cfg = match load_driver_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load driver config: {}. Using defaults.", e);
            DriverConfig::default_for_current_user()
        }
    };

    let path_mapper = PathMapper::from_config(
        cwd.clone(),
        driver_cfg.clone(),
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
            save_to_driver,
            save_to_library,
            browse_wsl_directory,
            get_setup_status,
            run_initial_setup,
            mark_setup_complete,
            compile_lpc,
            run_lpc,
            build_driver_ui,
            test_driver_connection,
            get_driver_config,
            save_driver_config_cmd,
            validate_and_diagnose_wsl,
            check_ollama_installed,
            check_ollama_running,
            start_ollama_server,
            pull_ollama_model,
            list_ollama_models,
            setup_staging_directory,
            commands::staging::save_to_staging,
            commands::staging::list_staged_files,
            commands::staging::copy_staged_to_project,
            commands::staging::clear_staging
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
