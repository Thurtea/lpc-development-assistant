use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DriverConfig {
    pub wsl_username: String,
    pub wsl_driver_root: String,
    pub wsl_library_root: String,
    pub driver_output_dir: Option<String>,
    pub library_output_dir: Option<String>,
    pub ollama_model: Option<String>,
    pub ollama_provider: Option<String>,
    pub ollama_enabled_tools: Vec<String>,
    pub staging_directory: Option<String>,
    pub setup_complete: bool,
}

impl DriverConfig {
    pub fn wsl_driver_path(&self) -> String {
        self.wsl_driver_root.clone()
    }

    pub fn wsl_library_path(&self) -> String {
        self.wsl_library_root.clone()
    }
}

impl DriverConfig {
    pub fn default_for_current_user() -> Self {
        DriverConfig::default()
    }
}

impl Default for DriverConfig {
    fn default() -> Self {
        let user = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_else(|_| "thurtea".to_string());
        let driver = format!("/home/{}/amlp-driver", user);
        let library = format!("/home/{}/amlp-library", user);
        Self {
            wsl_username: user.clone(),
            wsl_driver_root: driver.clone(),
            wsl_library_root: library.clone(),
            // Default output dirs per request
            driver_output_dir: Some("/home/thurtea/amlp-driver/src".to_string()),
            library_output_dir: Some("/home/thurtea/amlp-library/std".to_string()),
            ollama_model: Some("qwen2.5-coder:7b".to_string()),
            ollama_provider: Some("local".to_string()),
            ollama_enabled_tools: Vec::new(),
            staging_directory: None,
            setup_complete: false,
        }
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
    // Ensure the output dir fields default to the respective WSL roots if not present
    let final_cfg = DriverConfig {
        wsl_username: cfg.wsl_username,
        wsl_driver_root: cfg.wsl_driver_root.clone(),
        wsl_library_root: cfg.wsl_library_root.clone(),
        driver_output_dir: cfg.driver_output_dir.or(Some(cfg.wsl_driver_root.clone())),
        library_output_dir: cfg.library_output_dir.or(Some(cfg.wsl_library_root.clone())),
        ollama_model: cfg.ollama_model,
        ollama_provider: cfg.ollama_provider,
        ollama_enabled_tools: cfg.ollama_enabled_tools,
        staging_directory: cfg.staging_directory,
        setup_complete: cfg.setup_complete,
    };

    Ok(final_cfg)
}

pub fn save_driver_config(cfg: &DriverConfig) -> Result<(), String> {
    let path = match config_path() {
        Some(p) => p,
        None => return Err("Cannot determine config directory".into()),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // When saving, ensure the optional output dir fields are present by defaulting
    // them to the configured WSL roots if they are None.
    let cfg_to_save = DriverConfig {
        ollama_model: cfg.ollama_model.clone(),
        ollama_provider: cfg.ollama_provider.clone(),
        ollama_enabled_tools: cfg.ollama_enabled_tools.clone(),
        staging_directory: cfg.staging_directory.clone(),
        setup_complete: cfg.setup_complete,
        wsl_username: cfg.wsl_username.clone(),
        wsl_driver_root: cfg.wsl_driver_root.clone(),
        wsl_library_root: cfg.wsl_library_root.clone(),
        driver_output_dir: cfg.driver_output_dir.clone().or(Some(cfg.wsl_driver_root.clone())),
        library_output_dir: cfg.library_output_dir.clone().or(Some(cfg.wsl_library_root.clone())),
    };

    let raw = serde_json::to_string_pretty(&cfg_to_save).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| e.to_string())?;
    Ok(())
}
