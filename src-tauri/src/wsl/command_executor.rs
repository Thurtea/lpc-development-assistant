use anyhow::Result;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Event payloads preserved for future streaming functionality
pub enum CommandEvent {
    StdoutLine(String),
    StderrLine(String),
    Exit(i32),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

pub struct WslExecutor {
    pub workdir: Option<String>,
    pub timeout: Option<Duration>,
}

#[allow(dead_code)] // Methods preserved for future timeout and configuration features
impl WslExecutor {
    pub fn new(workdir: impl Into<String>) -> Self {
        WslExecutor {
            workdir: Some(workdir.into()),
            timeout: Some(Duration::from_secs(300)),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn execute(&self, command: &str) -> Result<CommandOutput> {
        let result = run_wsl_command(command, self.workdir.as_deref(), |_event| {});

        if let Some(timeout) = self.timeout {
            tokio::time::timeout(timeout, result)
                .await
                .map_err(|_| anyhow::anyhow!("Command timed out after {:?}", timeout))?
        } else {
            result.await
        }
    }
}

pub async fn run_wsl_command(
    command: &str,
    workdir: Option<&str>,
    mut on_event: impl FnMut(CommandEvent),
) -> Result<CommandOutput> {
    let full_command = if let Some(wd) = workdir {
        format!("cd {} && {}", shell_escape_path(wd), command)
    } else {
        command.to_string()
    };

    let mut child = Command::new("wsl.exe")
        .args(["-e", "bash", "-lc", &full_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("Failed to open stderr"))?;

    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();
    let mut stdout_line = String::new();
    let mut stderr_line = String::new();

    let mut stdout_done = false;
    let mut stderr_done = false;

    while !stdout_done || !stderr_done {
        tokio::select! {
            res = stdout_reader.read_line(&mut stdout_line), if !stdout_done => {
                match res {
                    Ok(0) => stdout_done = true,
                    Ok(_) => {
                        let trimmed = stdout_line.trim_end().to_string();
                        if !trimmed.is_empty() {
                            on_event(CommandEvent::StdoutLine(trimmed.clone()));
                            stdout_lines.push(trimmed);
                        }
                        stdout_line.clear();
                    }
                    Err(e) => return Err(anyhow::anyhow!("Error reading stdout: {}", e)),
                }
            }
            res = stderr_reader.read_line(&mut stderr_line), if !stderr_done => {
                match res {
                    Ok(0) => stderr_done = true,
                    Ok(_) => {
                        let trimmed = stderr_line.trim_end().to_string();
                        if !trimmed.is_empty() {
                            on_event(CommandEvent::StderrLine(trimmed.clone()));
                            stderr_lines.push(trimmed);
                        }
                        stderr_line.clear();
                    }
                    Err(e) => return Err(anyhow::anyhow!("Error reading stderr: {}", e)),
                }
            }
        }
    }

    let exit_status = child.wait().await?;
    let exit_code = exit_status.code();
    on_event(CommandEvent::Exit(exit_code.unwrap_or(-1)));

    Ok(CommandOutput {
        success: exit_code == Some(0),
        exit_code,
        stdout: stdout_lines,
        stderr: stderr_lines,
    })
}

fn shell_escape_path(path: &str) -> String {
    if path.contains(' ') || path.contains('\'') || path.contains('"') {
        format!("'{}'", path.replace('\'', "'\\''"))
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape_path("/simple/path"), "/simple/path");
        assert_eq!(shell_escape_path("/path with spaces"), "'/path with spaces'");
        assert_eq!(shell_escape_path("/path's/test"), "'/path'\\''s/test'");
    }
}
