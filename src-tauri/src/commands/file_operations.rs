use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

use crate::wsl::PathMapper;
use crate::config::{DriverConfig, load_driver_config};

const PROTECTED_PATTERNS: &[&str] = &[
    ".backup", "Makefile", "README.md", ".md",
    "master.c", "simul_efun.c", ".git", "logs", "data", "save"
];

fn is_protected_file(path: &str) -> bool {
    PROTECTED_PATTERNS.iter().any(|pattern| {
        if pattern.starts_with('.') && pattern.len() > 1 {
            path.ends_with(pattern)
        } else if pattern.contains('/') {
            path.contains(pattern)
        } else {
            path.split('/').next_back().unwrap_or("") == *pattern
        }
    })
}

fn shell_escape_single(s: &str) -> String {
    // Escape single quotes in bash by ending the single-quoted string,
    // inserting an escaped single quote, then reopening single quotes.
    // Example: abc'd -> 'abc'\''d'
    let esc = s.replace("'", "'\\''");
    format!("'{}'", esc)
}

fn build_path_mapper() -> Result<PathMapper, String> {
    let cfg = load_driver_config().unwrap_or_else(|_| DriverConfig::default());
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    Ok(PathMapper::new(cwd, cfg.wsl_driver_root.clone(), cfg.wsl_library_root.clone()))
}

fn get_default_wsl_distro() -> Result<String, String> {
    let out = Command::new("wsl.exe")
        .args(["-l", "-v"])
        .output()
        .map_err(|e| format!("Failed to run wsl.exe to list distros: {}", e))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("wsl.exe -l -v failed: exit {:?}; stderr: {}", out.status.code(), stderr));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    // Lines look like:  NAME   STATE   VERSION  or '* Ubuntu Running 2'
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('*') {
            let name = trimmed.trim_start_matches('*').split_whitespace().next().unwrap_or("Ubuntu").to_string();
            return Ok(name);
        }
    }

    for line in stdout.lines() {
        let t = line.trim();
        if t.is_empty() { continue; }
        if t.starts_with("NAME") || t.starts_with("--") { continue; }
        let name = t.split_whitespace().next().unwrap_or("Ubuntu").to_string();
        return Ok(name);
    }

    Ok("Ubuntu".to_string())
}

#[tauri::command]
pub fn save_to_driver(filename: String, contents: String, subdirectory: Option<String>) -> Result<String, String> {
    let cfg = load_driver_config().unwrap_or_else(|_| DriverConfig::default());
    let mut base = cfg.driver_output_dir.clone().unwrap_or(cfg.wsl_driver_root.clone());

    if let Some(s) = subdirectory {
        if s.starts_with('/') {
            // already a WSL path
            base = s;
        } else if Path::new(&s).is_absolute() {
            if let Ok(pm) = build_path_mapper() {
                if let Some(mapped) = pm.to_wsl_driver(Path::new(&s)) {
                    base = mapped;
                } else {
                    base = format!("{}/{}", base.trim_end_matches('/'), s.trim_start_matches('/'));
                }
            }
        } else {
            base = format!("{}/{}", base.trim_end_matches('/'), s.trim_end_matches('/'));
        }
    }

    let full_path = format!("{}/{}", base.trim_end_matches('/'), filename);

    // Protection: prevent accidental overwrite of important files
    if is_protected_file(&full_path) {
        return Err(format!("Cannot overwrite protected file: {}. See /home/thurtea/amlp-driver/WINDOWS_FILE_PLACEMENT_GUIDE.md for details.", full_path));
    }

    let distro = match get_default_wsl_distro() {
        Ok(d) => d,
        Err(e) => return Err(format!("WSL detection failed: {}; attempted path: {}", e, full_path)),
    };

    let mkdir_esc = shell_escape_single(&base);
    let file_esc = shell_escape_single(&full_path);
    // Atomic write: write to temp file then move into place and set permissions
    let write_cmd = format!(
        "mkdir -p {} && cat > {}.tmp && mv {}.tmp {} && chmod 644 {}",
        mkdir_esc, file_esc, file_esc, file_esc, file_esc
    );

    let mut child = Command::new("wsl.exe")
        .args(["-d", &distro, "--", "bash", "-lc", &write_cmd])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn wsl (distro={}): {}; attempted path: {}", distro, e, full_path))?;

    // Write contents to stdin, then collect output
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(contents.as_bytes()) {
            return Err(format!("Failed to write to wsl stdin for {}: {}", full_path, e));
        }
    } else {
        return Err(format!("Failed to open stdin to wsl process; attempted path: {}", full_path));
    }

    let output = child.wait_with_output().map_err(|e| format!("Failed waiting for wsl process: {}; attempted path: {}", e, full_path))?;

    if output.status.success() {
        return Ok(full_path);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    // Classify common failures
    if stderr.to_lowercase().contains("not found") || stderr.to_lowercase().contains("no such file") {
        return Err(format!("Path invalid or command not found in WSL (distro={}): {}; stderr: {}; attempted path: {}", distro, write_cmd, stderr, full_path));
    }
    if stderr.to_lowercase().contains("permission denied") || stderr.to_lowercase().contains("access is denied") {
        return Err(format!("Permission denied when writing to WSL path (distro={}): {}; stderr: {}; attempted path: {}", distro, write_cmd, stderr, full_path));
    }
    // Distro errors
    if stderr.to_lowercase().contains("wsl") && stderr.to_lowercase().contains("error") {
        return Err(format!("WSL/distro error (distro={}): {}; stderr: {}; attempted path: {}", distro, write_cmd, stderr, full_path));
    }

    Err(format!("WSL command failed (distro={}): exit {:?}; stderr: {}; attempted path: {}", distro, output.status.code(), stderr, full_path))
}

#[tauri::command]
pub fn save_to_library(filename: String, contents: String, subdirectory: Option<String>) -> Result<String, String> {
    let cfg = load_driver_config().unwrap_or_else(|_| DriverConfig::default());
    let mut base = cfg.library_output_dir.clone().unwrap_or(cfg.wsl_library_root.clone());

    if let Some(s) = subdirectory {
        if s.starts_with('/') {
            base = s;
        } else if Path::new(&s).is_absolute() {
            if let Ok(pm) = build_path_mapper() {
                if let Some(mapped) = pm.to_wsl_library(Path::new(&s)) {
                    base = mapped;
                } else {
                    base = format!("{}/{}", base.trim_end_matches('/'), s.trim_start_matches('/'));
                }
            }
        } else {
            base = format!("{}/{}", base.trim_end_matches('/'), s.trim_end_matches('/'));
        }
    }

    let full_path = format!("{}/{}", base.trim_end_matches('/'), filename);

    // Protection: prevent accidental overwrite of important files
    if is_protected_file(&full_path) {
        return Err(format!("Cannot overwrite protected file: {}. See /home/thurtea/amlp-driver/WINDOWS_FILE_PLACEMENT_GUIDE.md for details.", full_path));
    }

    let distro = match get_default_wsl_distro() {
        Ok(d) => d,
        Err(e) => return Err(format!("WSL detection failed: {}; attempted path: {}", e, full_path)),
    };

    let mkdir_esc = shell_escape_single(&base);
    let file_esc = shell_escape_single(&full_path);
    // Atomic write: write to temp file then move into place and set permissions
    let write_cmd = format!(
        "mkdir -p {} && cat > {}.tmp && mv {}.tmp {} && chmod 644 {}",
        mkdir_esc, file_esc, file_esc, file_esc, file_esc
    );

    let mut child = Command::new("wsl.exe")
        .args(["-d", &distro, "--", "bash", "-lc", &write_cmd])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn wsl (distro={}): {}; attempted path: {}", distro, e, full_path))?;

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(contents.as_bytes()) {
            return Err(format!("Failed to write to wsl stdin for {}: {}", full_path, e));
        }
    } else {
        return Err(format!("Failed to open stdin to wsl process; attempted path: {}", full_path));
    }

    let output = child.wait_with_output().map_err(|e| format!("Failed waiting for wsl process: {}; attempted path: {}", e, full_path))?;

    if output.status.success() {
        return Ok(full_path);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if stderr.to_lowercase().contains("not found") || stderr.to_lowercase().contains("no such file") {
        return Err(format!("Path invalid or command not found in WSL (distro={}): {}; stderr: {}; attempted path: {}", distro, write_cmd, stderr, full_path));
    }
    if stderr.to_lowercase().contains("permission denied") || stderr.to_lowercase().contains("access is denied") {
        return Err(format!("Permission denied when writing to WSL path (distro={}): {}; stderr: {}; attempted path: {}", distro, write_cmd, stderr, full_path));
    }
    if stderr.to_lowercase().contains("wsl") && stderr.to_lowercase().contains("error") {
        return Err(format!("WSL/distro error (distro={}): {}; stderr: {}; attempted path: {}", distro, write_cmd, stderr, full_path));
    }

    Err(format!("WSL command failed (distro={}): exit {:?}; stderr: {}; attempted path: {}", distro, output.status.code(), stderr, full_path))
}
