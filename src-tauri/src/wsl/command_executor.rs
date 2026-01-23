use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub enum CommandEvent {
    Stdout(String),
    Stderr(String),
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WslExecutor {
    workdir: Option<String>,
}

impl WslExecutor {
    pub fn new(workdir: Option<String>) -> Self {
        Self { workdir }
    }

    pub async fn execute(&self, command: &str) -> Result<CommandOutput> {
        run_wsl_command(command, self.workdir.as_deref(), |_ev| {}).await
    }
}

pub async fn run_wsl_command(
    command: &str,
    workdir: Option<&str>,
    mut on_event: impl FnMut(CommandEvent),
) -> Result<CommandOutput> {
    let bash = match workdir {
        Some(dir) => format!("cd {} && {}", dir, command),
        None => command.to_string(),
    };

    let mut child = Command::new("wsl.exe")
        .arg("-e")
        .arg("bash")
        .arg("-lc")
        .arg(bash)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn wsl command")?;

    let stdout = child
        .stdout
        .take()
        .context("child missing stdout pipe")?;
    let stderr = child
        .stderr
        .take()
        .context("child missing stderr pipe")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    while !stdout_done || !stderr_done {
        tokio::select! {
            line = stdout_reader.next_line(), if !stdout_done => {
                match line.context("read stdout line")? {
                    Some(l) => {
                        on_event(CommandEvent::Stdout(l.clone()));
                        stdout_lines.push(l);
                    }
                    None => stdout_done = true,
                }
            },
            line = stderr_reader.next_line(), if !stderr_done => {
                match line.context("read stderr line")? {
                    Some(l) => {
                        on_event(CommandEvent::Stderr(l.clone()));
                        stderr_lines.push(l);
                    }
                    None => stderr_done = true,
                }
            }
        }
    }

    let status = child.wait().await.context("failed to wait on child")?;
    let exit_code = status.code();

    Ok(CommandOutput {
        exit_code,
        stdout: stdout_lines,
        stderr: stderr_lines,
    })
}
