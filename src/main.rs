use std::path::PathBuf;

mod ollama_client;
mod context_manager;

use ollama_client::OllamaClient;
use context_manager::ContextManager;
use tauri::Emitter;

#[tauri::command]
async fn ask_ollama(model: String, question: String, context_type: Option<String>) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cm = ContextManager::new(PathBuf::from(cwd));
    cm.ensure_templates_exist().map_err(|e| e.to_string())?;

    let mut prompt = String::new();
    match context_type.as_deref() {
        Some("driver") => prompt.push_str(&cm.load_driver_context().map_err(|e| e.to_string())?),
        Some("mudlib") => prompt.push_str(&cm.load_mudlib_context().map_err(|e| e.to_string())?),
        Some("efuns") => prompt.push_str(&cm.load_efuns_context().map_err(|e| e.to_string())?),
        Some("references") => prompt.push_str(&cm.load_reference_sources_context().map_err(|e| e.to_string())?),
        _ => {}
    }

    prompt.push_str("\n\nQUESTION:\n");
    prompt.push_str(&question);

    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    client.generate(&model, &prompt).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_models() -> Result<Vec<String>, String> {
    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    client.list_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
fn extract_references() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cm = ContextManager::new(PathBuf::from(cwd));
    cm.extract_archives().map_err(|e| e.to_string())?;
    Ok("Extraction completed".to_string())
}

#[tauri::command]
fn search_examples(keyword: String) -> Result<Vec<String>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cm = ContextManager::new(PathBuf::from(cwd));
    let results = cm.search_code_examples(&keyword);
    Ok(results.iter().map(|p| p.display().to_string()).collect())
}

#[tauri::command]
fn save_response(filename: String, contents: String) -> Result<String, String> {
    // Save to provided filename (relative paths will be anchored at cwd)
    let path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join(filename);
    std::fs::write(&path, contents).map_err(|e| e.to_string())?;
    Ok(format!("Saved to {}", path.display()))
}

// New template-related tauri commands
#[tauri::command]
async fn get_template(name: String) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cm = ContextManager::new(PathBuf::from(cwd));
    cm.ensure_templates_exist().map_err(|e| e.to_string())?;

    match name.as_str() {
        "simul_efun" => cm.load_simul_efun_context().map_err(|e| e.to_string()),
        "master_api" => cm.load_master_api_context().map_err(|e| e.to_string()),
        "socket_api" => cm.load_socket_api_context().map_err(|e| e.to_string()),
        "comm" => cm.load_comm_context().map_err(|e| e.to_string()),
        "backend" => cm.load_backend_context().map_err(|e| e.to_string()),
        other => cm.load_template_by_filename(other).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
async fn list_templates() -> Result<Vec<String>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cm = ContextManager::new(PathBuf::from(cwd));
    cm.ensure_templates_exist().map_err(|e| e.to_string())?;

    let mut names = Vec::new();
    for entry in std::fs::read_dir(cm.templates_path).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        if let Some(name) = e.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

#[cfg(feature = "with_tauri")]
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ask_ollama,
            list_models,
            extract_references,
            search_examples,
            save_response,
            ask_ollama_stream,
            get_template,
            list_templates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn ask_ollama_stream(window: tauri::Window, model: String, question: String, context_type: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cm = ContextManager::new(PathBuf::from(cwd));
    cm.ensure_templates_exist().map_err(|e| e.to_string())?;

    let mut prompt = String::new();
    match context_type.as_deref() {
        Some("driver") => prompt.push_str(&cm.load_driver_context().map_err(|e| e.to_string())?),
        Some("mudlib") => prompt.push_str(&cm.load_mudlib_context().map_err(|e| e.to_string())?),
        Some("efuns") => prompt.push_str(&cm.load_efuns_context().map_err(|e| e.to_string())?),
        Some("references") => prompt.push_str(&cm.load_reference_sources_context().map_err(|e| e.to_string())?),
        _ => {}
    }

    prompt.push_str("\n\nQUESTION:\n");
    prompt.push_str(&question);

    let client = OllamaClient::new().map_err(|e| e.to_string())?;
    let stream = client.generate_stream(&model, &prompt);
    let win = window.clone();

    tauri::async_runtime::spawn(async move {
        use tokio_stream::StreamExt;
        let mut s = stream;
        while let Some(item) = s.next().await {
            match item {
                Ok(tok) => {
                    let _ = win.emit("ollama-token", tok);
                }
                Err(e) => {
                    let _ = win.emit("ollama-stream-error", e.to_string());
                    break;
                }
            }
        }
        let _ = win.emit("ollama-stream-done", ());
    });

    Ok(())
}

#[cfg(not(feature = "with_tauri"))]
fn main() {
    println!("LPC Dev Assistant (tauri disabled). Build with --features with_tauri to enable the GUI.");
}
