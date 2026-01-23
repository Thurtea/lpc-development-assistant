use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathMapper {
    windows_root: PathBuf,
    wsl_driver_root: String,
    wsl_library_root: String,
}

impl PathMapper {
    pub fn new(windows_root: PathBuf, wsl_driver_root: String, wsl_library_root: String) -> Self {
        Self {
            windows_root,
            wsl_driver_root,
            wsl_library_root,
        }
    }

    pub fn wsl_driver_root(&self) -> &str {
        &self.wsl_driver_root
    }

    pub fn wsl_library_root(&self) -> &str {
        &self.wsl_library_root
    }

    pub fn to_wsl_driver(&self, windows_path: &Path) -> Option<String> {
        if windows_path.is_absolute() && windows_path.starts_with(&self.windows_root) {
            let rel = windows_path.strip_prefix(&self.windows_root).ok()?;
            return Some(self.join_wsl(&self.wsl_driver_root, rel));
        }
        if let Some(s) = windows_path.to_str() {
            if s.starts_with('/') {
                return Some(s.to_string());
            }
        }
        None
    }

    pub fn to_wsl_library(&self, windows_path: &Path) -> Option<String> {
        if windows_path.is_absolute() && windows_path.starts_with(&self.windows_root) {
            let rel = windows_path.strip_prefix(&self.windows_root).ok()?;
            return Some(self.join_wsl(&self.wsl_library_root, rel));
        }
        if let Some(s) = windows_path.to_str() {
            if s.starts_with('/') {
                return Some(s.to_string());
            }
        }
        None
    }

    fn join_wsl(&self, root: &str, rel: &Path) -> String {
        let mut parts = vec![root.trim_end_matches('/').to_string()];
        for comp in rel.components() {
            let seg = comp.as_os_str().to_string_lossy().replace('\\', "/");
            if !seg.is_empty() {
                parts.push(seg);
            }
        }
        parts.join("/")
    }
}
