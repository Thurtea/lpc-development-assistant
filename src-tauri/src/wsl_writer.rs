use std::ffi::OsStr;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// Return true if the given wsl path should be protected from automatic overwrites.
pub fn is_protected_file(path: &str) -> bool {
    let p = Path::new(path);

    // Check for backup suffix
    if let Some(ext) = p.extension().and_then(OsStr::to_str) {
        if ext == "backup" {
            return true;
        }
        // any markdown files
        if ext.eq_ignore_ascii_case("md") {
            return true;
        }
    }

    // File name checks
    if let Some(name) = p.file_name().and_then(OsStr::to_str) {
        let lower = name.to_ascii_lowercase();
        if lower == "makefile" || lower == "readme.md" || lower == "readme" {
            return true;
        }
        if lower == "master.c" {
            return true;
        }
    }

    // Paths that start with these protected directories
    let comps: Vec<String> = p
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str().map(|s| s.to_string()),
            _ => None,
        })
        .collect();

    // join to normalized path string
    let joined = comps.join("/");

    // protected directories and special files
    if joined.starts_with(".git") || joined.starts_with("git/") {
        return true;
    }
    if joined.starts_with("logs/") || joined.starts_with("data/") || joined.starts_with("save/") {
        return true;
    }
    if joined.contains("secure/simul_efun.c") || joined.ends_with("secure/simul_efun.c") {
        return true;
    }

    false
}

fn escape_single_quotes(s: &str) -> String {
    // Escape single quotes for embedding inside single-quoted shell literal:
    // 'a' -> 'a' ; "a'b" -> 'a'\''b'
    s.replace('\'','"'\\''")
}

/// Write a file atomically into WSL by invoking `wsl.exe -d <distro> -- bash -lc '<commands>'`
/// The function performs: mkdir -p <parent> && cat > <file>.tmp && mv <file>.tmp <file> && chmod 644 <file>
/// Content is piped to the child's stdin.
pub fn write_file_atomic(distro: &str, wsl_path: &str, content: &str) -> Result<String, String> {
    // Basic validation of inputs
    if wsl_path.is_empty() {
        return Err("empty target path".to_string());
    }

    let target = Path::new(wsl_path);
    let parent = target
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "/".to_string());

    // Escape single quotes in paths for safe single-quoted bash literal
    let esc_parent = escape_single_quotes(&parent);
    let esc_target = escape_single_quotes(&wsl_path);
    let tmp_target = format!("{}.tmp", wsl_path);
    let esc_tmp = escape_single_quotes(&tmp_target);

    // Build the shell command to run on the WSL side
    let shell_cmd = format!(
        "mkdir -p '{}' && cat > '{}' && mv '{}' '{}' && chmod 644 '{}'",
        esc_parent, esc_tmp, esc_tmp, esc_target, esc_target
    );

    // Spawn wsl.exe and write contents to its stdin
    let mut child = Command::new("wsl.exe")
        .arg("-d")
        .arg(distro)
        .arg("--")
        .arg("bash")
        .arg("-lc")
        .arg(&shell_cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn wsl.exe for {}: {}", wsl_path, e))?;

    // Write content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("failed to write content to wsl stdin for {}: {}", wsl_path, e))?;
    } else {
        return Err(format!("failed to open stdin for wsl process for {}", wsl_path));
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("waiting for wsl process failed for {}: {}", wsl_path, e))?;

    if output.status.success() {
        Ok(wsl_path.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "wsl write failed for {}: status={} stdout='{}' stderr='{}'",
            wsl_path,
            output.status,
            stdout,
            stderr
        ))
    }
}

/// Validate a target path for allowed placement and forbid path traversal.
pub fn validate_path(path: &str, file_type: &str) -> Result<(), String> {
    let p = Path::new(path);

    // Disallow parent directory components
    for comp in p.components() {
        if let Component::ParentDir = comp {
            return Err(format!("path contains parent-dir traversal: {}", path));
        }
    }

    match file_type {
        "driver_source" | "driver_header" => {
            if !path.starts_with("/home/thurtea/amlp-driver/src/") {
                return Err(format!("driver files must be under /home/thurtea/amlp-driver/src/: {}", path));
            }
        }
        "library_std" => {
            if !path.starts_with("/home/thurtea/amlp-library/std/") {
                return Err(format!("std library files must be under /home/thurtea/amlp-library/std/: {}", path));
            }
        }
        "library_secure" => {
            if !path.starts_with("/home/thurtea/amlp-library/secure/") {
                return Err(format!("secure library files must be under /home/thurtea/amlp-library/secure/: {}", path));
            }
        }
        "library_domain" => {
            if !path.starts_with("/home/thurtea/amlp-library/domains/") {
                return Err(format!("domain library files must be under /home/thurtea/amlp-library/domains/: {}", path));
            }
        }
        other => return Err(format!("unknown file_type for validation: {}", other)),
    }

    // Disallow nulls and control characters
    if path.chars().any(|c| c == '\0' || c == '\r' || c == '\n') {
        return Err(format!("path contains invalid characters: {}", path));
    }

    Ok(())
}

/// Save a driver file (either .c or .h) into the driver src directory.
/// `filename` should be a plain filename (no directory separators) like `new_module.c`.
pub fn save_driver_file(
    distro: &str,
    filename: &str,
    content: &str,
    is_header: bool,
) -> Result<String, String> {
    // Disallow path separators in filename for this helper
    if filename.contains('/') || filename.contains('\\') {
        return Err(format!("filename must not include path separators: {}", filename));
    }

    if is_header && !filename.ends_with(".h") {
        return Err(format!("header filename must end with .h: {}", filename));
    }
    if !is_header && !filename.ends_with(".c") {
        return Err(format!("source filename must end with .c: {}", filename));
    }

    let full = format!("/home/thurtea/amlp-driver/src/{}", filename);

    // Validate and protected checks
    validate_path(&full, if is_header { "driver_header" } else { "driver_source" })?;
    if is_protected_file(&full) {
        return Err(format!("refusing to overwrite protected file: {}", full));
    }

    write_file_atomic(distro, &full, content)
}

/// Save a library file. For `category` use:
/// - "std" -> writes to /home/thurtea/amlp-library/std/<filename>
/// - "secure" -> writes to /home/thurtea/amlp-library/secure/<filename>
/// - "domain" -> `domain` must be Some and may include subpaths like "tutorial/rooms"
///              -> writes to /home/thurtea/amlp-library/domains/<domain>/<filename>
pub fn save_library_file(
    distro: &str,
    category: &str,
    domain: Option<&str>,
    filename: &str,
    content: &str,
) -> Result<String, String> {
    if !filename.ends_with(".c") {
        return Err(format!("library filenames must end with .c: {}", filename));
    }

    let full = match category {
        "std" => format!("/home/thurtea/amlp-library/std/{}", filename),
        "secure" => format!("/home/thurtea/amlp-library/secure/{}", filename),
        "domain" => {
            let domain = domain.ok_or_else(|| "domain must be provided for category=domain".to_string())?;
            // Prevent traversal in the provided domain string
            if domain.contains("..") || domain.contains('\0') {
                return Err(format!("invalid domain path: {}", domain));
            }
            format!("/home/thurtea/amlp-library/domains/{}/{}", domain.trim_start_matches('/'), filename)
        }
        other => return Err(format!("unknown library category: {}", other)),
    };

    // Determine file_type for validation
    let file_type = if category == "std" {
        "library_std"
    } else if category == "secure" {
        "library_secure"
    } else {
        "library_domain"
    };

    validate_path(&full, file_type)?;
    if is_protected_file(&full) {
        return Err(format!("refusing to overwrite protected file: {}", full));
    }

    write_file_atomic(distro, &full, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_protected() {
        assert!(is_protected_file("/home/thurtea/amlp-driver/src/foo.c.backup"));
        assert!(is_protected_file("/home/thurtea/amlp-library/master.c"));
        assert!(is_protected_file(".git/config"));
        assert!(is_protected_file("logs/runtime.log"));
    }

    #[test]
    fn test_validate_path_ok() {
        assert!(validate_path("/home/thurtea/amlp-driver/src/new.c", "driver_source").is_ok());
        assert!(validate_path("/home/thurtea/amlp-library/std/object.c", "library_std").is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("/home/thurtea/amlp-driver/src/../etc/passwd", "driver_source").is_err());
    }
}
