use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use chrono::Local;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedFile {
    pub relative_path: String,      // e.g., "lib/domains/forest/rooms/clearing.c"
    pub full_path: String,           // Full path in staging
    pub target_project: String,      // "driver" or "library"
    pub content: String,             // File content
    pub status: String,              // "pending", "verified", "copied"
}

#[tauri::command]
pub async fn save_to_staging(
    staging_dir: String,
    relative_path: String,
    content: String,
    _target_project: String,
) -> Result<String, String> {
    let staging_path = PathBuf::from(&staging_dir);
    
    // Create staging directory if it doesn't exist
    if !staging_path.exists() {
        fs::create_dir_all(&staging_path).map_err(|e| format!("Failed to create staging directory: {}", e))?;
    }
    
    // Build full file path
    let file_path = staging_path.join(&relative_path);
    
    // Create parent directories as needed
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }
    
    // Write file content
    fs::write(&file_path, &content).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(format!("Saved to staging: {}", relative_path))
}

#[tauri::command]
pub async fn list_staged_files(staging_dir: String) -> Result<Vec<StagedFile>, String> {
    let staging_path = PathBuf::from(&staging_dir);
    
    if !staging_path.exists() {
        return Ok(Vec::new());
    }
    
    let mut files = Vec::new();
    
    for entry in WalkDir::new(&staging_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let full_path = entry.path();
        let relative_path = full_path
            .strip_prefix(&staging_path)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        
        // Determine target project from path
        let target_project = if relative_path.starts_with("driver/") || relative_path.starts_with("lib/") {
            "driver".to_string()
        } else {
            "library".to_string()
        };
        
        // Read file content
        let content = fs::read_to_string(full_path)
            .map_err(|e| format!("Failed to read file {}: {}", relative_path, e))?;
        
        files.push(StagedFile {
            relative_path,
            full_path: full_path.to_string_lossy().to_string(),
            target_project,
            content,
            status: "pending".to_string(),
        });
    }
    
    Ok(files)
}

#[tauri::command]
pub async fn copy_staged_to_project(
    staged_file: StagedFile,
    driver_root: String,
    library_root: String,
    use_wsl: bool,
) -> Result<String, String> {
    // Determine target root
    let target_root = if staged_file.target_project == "driver" {
        &driver_root
    } else {
        &library_root
    };
    
    // Strip "driver/" or "lib/" prefix from relative_path
    let mut target_relative_path = staged_file.relative_path.clone();
    if target_relative_path.starts_with("driver/") {
        target_relative_path = target_relative_path.strip_prefix("driver/").unwrap().to_string();
    } else if target_relative_path.starts_with("lib/") {
        target_relative_path = target_relative_path.strip_prefix("lib/").unwrap().to_string();
    }
    
    if use_wsl {
        // Use WSL commands
        copy_via_wsl(target_root, &target_relative_path, &staged_file.content).await
    } else {
        // Direct file copy
        copy_direct(target_root, &target_relative_path, &staged_file.content)
    }
}

async fn copy_via_wsl(root: &str, relative_path: &str, content: &str) -> Result<String, String> {
    let target_path = format!("{}/{}", root.trim_end_matches('/'), relative_path);
    
    // Check if file exists and create backup if needed
    let check_cmd = format!("test -f {}", target_path);
    let check_output = tokio::process::Command::new("wsl.exe")
        .args(["-e", "bash", "-lc", &check_cmd])
        .output()
        .await
        .map_err(|e| format!("Failed to check file existence: {}", e))?;
    
    if check_output.status.success() {
        // File exists, create backup
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let backup_path = format!("{}.bak-{}", target_path, timestamp);
        let backup_cmd = format!("cp {} {}", target_path, backup_path);
        
        tokio::process::Command::new("wsl.exe")
            .args(["-e", "bash", "-lc", &backup_cmd])
            .output()
            .await
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    
    // Create parent directories
    let parent_dir = if let Some(idx) = relative_path.rfind('/') {
        format!("{}/{}", root.trim_end_matches('/'), &relative_path[..idx])
    } else {
        root.to_string()
    };
    
    let mkdir_cmd = format!("mkdir -p {}", parent_dir);
    tokio::process::Command::new("wsl.exe")
        .args(["-e", "bash", "-lc", &mkdir_cmd])
        .output()
        .await
        .map_err(|e| format!("Failed to create directories: {}", e))?;
    
    // Write file content using tee (to handle special characters safely)
    // First write to temp file, then move
    let temp_path = format!("/tmp/lpc_staging_{}", Local::now().timestamp());
    let write_cmd = format!("cat > {}", temp_path);
    
    let mut child = tokio::process::Command::new("wsl.exe")
        .args(["-e", "bash", "-lc", &write_cmd])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn write command: {}", e))?;
    
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(content.as_bytes()).await
            .map_err(|e| format!("Failed to write content: {}", e))?;
    }
    
    child.wait().await.map_err(|e| format!("Write command failed: {}", e))?;
    
    // Move temp file to target
    let mv_cmd = format!("mv {} {}", temp_path, target_path);
    tokio::process::Command::new("wsl.exe")
        .args(["-e", "bash", "-lc", &mv_cmd])
        .output()
        .await
        .map_err(|e| format!("Failed to move file: {}", e))?;
    
    Ok(format!("Copied to WSL project: {}", target_path))
}

fn copy_direct(root: &str, relative_path: &str, content: &str) -> Result<String, String> {
    let target_path = PathBuf::from(root).join(relative_path);
    
    // Create backup if file exists
    if target_path.exists() {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let backup_path = format!("{}.bak-{}", target_path.display(), timestamp);
        fs::copy(&target_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    
    // Create parent directories
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }
    
    // Write file content
    fs::write(&target_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(format!("Copied to project: {}", target_path.display()))
}

#[tauri::command]
pub async fn clear_staging(staging_dir: String) -> Result<String, String> {
    let staging_path = PathBuf::from(&staging_dir);
    
    if !staging_path.exists() {
        return Ok("Staging directory is already empty".to_string());
    }
    
    // Remove all files and subdirectories
    for entry in WalkDir::new(&staging_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        fs::remove_file(entry.path())
            .map_err(|e| format!("Failed to remove file: {}", e))?;
    }
    
    // Clean up empty directories
    for entry in WalkDir::new(&staging_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.path() != staging_path)
    {
        let _ = fs::remove_dir(entry.path()); // Ignore errors for non-empty dirs
    }
    
    Ok("Staging directory cleared".to_string())
}
