use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::wsl::{command_executor::WslExecutor, PathMapper};

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
        DriverPipeline {
            executor: WslExecutor::new(paths.wsl_driver_root().to_string()),
            paths,
        }
    }

    pub async fn compile(&self, file_path: &str, _on_event: impl FnMut(crate::wsl::command_executor::CommandEvent)) -> Result<CompileResult> {
        let safe_path = self.validate_and_escape_path(file_path)?;
        let command = format!("./build/driver compile {}", safe_path);

        let output = self.executor.execute(&command).await?;
        Ok(CompileResult {
            success: output.success,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub async fn ast(&self, file_path: &str, _on_event: impl FnMut(crate::wsl::command_executor::CommandEvent)) -> Result<CompileResult> {
        let safe_path = self.validate_and_escape_path(file_path)?;
        let command = format!("./build/driver ast {}", safe_path);

        let output = self.executor.execute(&command).await?;
        Ok(CompileResult {
            success: output.success,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub async fn bytecode(&self, file_path: &str, _on_event: impl FnMut(crate::wsl::command_executor::CommandEvent)) -> Result<CompileResult> {
        let safe_path = self.validate_and_escape_path(file_path)?;
        let command = format!("./build/driver bytecode {}", safe_path);

        let output = self.executor.execute(&command).await?;
        Ok(CompileResult {
            success: output.success,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub async fn build_ui(&self, _on_event: impl FnMut(crate::wsl::command_executor::CommandEvent)) -> Result<CompileResult> {
        let command = "make build-ui".to_string();

        let output = self.executor.execute(&command).await?;
        Ok(CompileResult {
            success: output.success,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub async fn test(&self, _on_event: impl FnMut(crate::wsl::command_executor::CommandEvent)) -> Result<CompileResult> {
        let command = "make test".to_string();

        let output = self.executor.execute(&command).await?;
        Ok(CompileResult {
            success: output.success,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    fn validate_and_escape_path(&self, path: &str) -> Result<String> {
        if path.starts_with('/') {
            return Ok(shell_escape(path));
        }

        let windows_path = std::path::PathBuf::from(path);
        let wsl_path = self.paths
            .to_wsl_driver(&windows_path)
            .or_else(|| self.paths.to_wsl_library(&windows_path))
            .ok_or_else(|| anyhow::anyhow!("Path '{}' is not within driver or library roots", path))?;

        Ok(shell_escape(&wsl_path))
    }
}

fn shell_escape(input: &str) -> String {
    if input.contains(' ') || input.contains('\'') || input.contains('"')
        || input.contains('$') || input.contains('`') || input.contains('\\') {
        format!("'{}'", input.replace('\'', "'\\''"))
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("simple.c"), "simple.c");
        assert_eq!(shell_escape("file with spaces.c"), "'file with spaces.c'");
        assert_eq!(shell_escape("file's.c"), "'file'\\''s.c'");
        assert_eq!(shell_escape("$(evil).c"), "'$(evil).c'");
        assert_eq!(shell_escape("`evil`.c"), "'`evil`.c'");
    }
}
