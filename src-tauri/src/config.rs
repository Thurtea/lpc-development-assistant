use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverConfig {
    pub wsl_username: String,
    pub wsl_driver_root: String,
    pub wsl_library_root: String,
}

impl DriverConfig {
    pub fn default_for_current_user() -> Self {
        let user = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_else(|_| "thurtea".to_string());
        Self {
            wsl_username: user.clone(),
            wsl_driver_root: format!("/home/{}/amlp-driver", user),
            wsl_library_root: format!("/home/{}/amlp-library", user),
        }
    }

    pub fn wsl_driver_path(&self) -> String {
        self.wsl_driver_root.clone()
    }

    pub fn wsl_library_path(&self) -> String {
        self.wsl_library_root.clone()
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d: PathBuf| d.join("lpc_dev_assistant_driver_config.json"))
}

pub fn load_driver_config() -> Result<DriverConfig, String> {
    let default = DriverConfig::default_for_current_user();
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(default),
    };

    if !path.exists() {
        // try to create parent dir
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // write default
        let _ = save_driver_config(&default);
        return Ok(default);
    }

    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let cfg: DriverConfig = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(cfg)
}

pub fn save_driver_config(cfg: &DriverConfig) -> Result<(), String> {
    let path = match config_path() {
        Some(p) => p,
        None => return Err("Cannot determine config directory".into()),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| e.to_string())?;
    Ok(())
}
