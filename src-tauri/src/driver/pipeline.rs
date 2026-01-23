use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::wsl::{run_wsl_command, CommandEvent, CommandOutput, PathMapper, WslExecutor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

pub struct DriverPipeline {
    pub executor: WslExecutor,
    paths: Arc<PathMapper>,
}

impl DriverPipeline {
    pub fn new(paths: Arc<PathMapper>) -> Self {
        let workdir = paths.wsl_driver_root().to_string();
        Self {
            executor: WslExecutor::new(Some(workdir)),
            paths,
        }
    }

    pub async fn compile(
        &self,
        file_path: &str,
        mut on_event: impl FnMut(CommandEvent),
    ) -> Result<CompileResult> {
        let wsl_path = self
            .paths
            .to_wsl_library(Path::new(file_path))
            .or_else(|| self.paths.to_wsl_driver(Path::new(file_path)))
            .ok_or_else(|| anyhow!("could not map file path to WSL"))?;
        let cmd = format!("./amlp-driver compile {}", shell_escape(&wsl_path));
        self.run(cmd, self.paths.wsl_driver_root(), on_event).await
    }

    pub async fn ast(&self, file_path: &str, mut on_event: impl FnMut(CommandEvent)) -> Result<CompileResult> {
        let wsl_path = self
            .paths
            .to_wsl_library(Path::new(file_path))
            .or_else(|| self.paths.to_wsl_driver(Path::new(file_path)))
            .ok_or_else(|| anyhow!("could not map file path to WSL"))?;
        let cmd = format!("./amlp-driver ast {}", shell_escape(&wsl_path));
        self.run(cmd, self.paths.wsl_driver_root(), on_event).await
    }

    pub async fn bytecode(
        &self,
        file_path: &str,
        mut on_event: impl FnMut(CommandEvent),
    ) -> Result<CompileResult> {
        let wsl_path = self
            .paths
            .to_wsl_library(Path::new(file_path))
            .or_else(|| self.paths.to_wsl_driver(Path::new(file_path)))
            .ok_or_else(|| anyhow!("could not map file path to WSL"))?;
        let cmd = format!("./amlp-driver bytecode {}", shell_escape(&wsl_path));
        self.run(cmd, self.paths.wsl_driver_root(), on_event).await
    }

    pub async fn build_ui(&self, mut on_event: impl FnMut(CommandEvent)) -> Result<CompileResult> {
        self.run("make build-ui".to_string(), self.paths.wsl_driver_root(), on_event)
            .await
    }

    pub async fn test(&self, mut on_event: impl FnMut(CommandEvent)) -> Result<CompileResult> {
        self.run("make test".to_string(), self.paths.wsl_driver_root(), on_event)
            .await
    }

    async fn run(
        &self,
        command: String,
        workdir: &str,
        mut on_event: impl FnMut(CommandEvent),
    ) -> Result<CompileResult> {
        let out: CommandOutput = run_wsl_command(&command, Some(workdir), |ev| on_event(ev)).await?;
        let success = out.exit_code.unwrap_or(1) == 0;
        Ok(CompileResult {
            success,
            exit_code: out.exit_code,
            stdout: out.stdout,
            stderr: out.stderr,
        })
    }
}

fn shell_escape(input: &str) -> String {
    if input.chars().all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '.' || c == '-') {
        return input.to_string();
    }
    let mut out = String::from("'");
    for ch in input.chars() {
        if ch == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}
