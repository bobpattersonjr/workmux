use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::{Command, Output};

/// A builder for executing shell commands with unified error handling
pub struct Cmd<'a> {
    command: &'a str,
    args: Vec<&'a str>,
    workdir: Option<&'a Path>,
}

impl<'a> Cmd<'a> {
    /// Create a new command builder
    pub fn new(command: &'a str) -> Self {
        Self {
            command,
            args: Vec::new(),
            workdir: None,
        }
    }

    /// Add a single argument
    pub fn arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }

    /// Add multiple arguments
    pub fn args(mut self, args: &[&'a str]) -> Self {
        self.args.extend_from_slice(args);
        self
    }

    /// Set the working directory for the command
    pub fn workdir(mut self, path: &'a Path) -> Self {
        self.workdir = Some(path);
        self
    }

    /// Execute the command and return the output
    /// Returns an error if the command fails (non-zero exit code)
    pub fn run(self) -> Result<Output> {
        let mut cmd = Command::new(self.command);
        if let Some(dir) = self.workdir {
            cmd.current_dir(dir);
        }
        let output = cmd.args(&self.args).output().with_context(|| {
            format!(
                "Failed to execute command: {} {}",
                self.command,
                self.args.join(" ")
            )
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Command failed: {} {}\n{}",
                self.command,
                self.args.join(" "),
                stderr.trim()
            ));
        }
        Ok(output)
    }

    /// Execute the command and return stdout as a trimmed string
    pub fn run_and_capture_stdout(self) -> Result<String> {
        let output = self.run()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Execute the command, returning Ok(true) if it succeeds, Ok(false) if it fails
    /// This is useful for commands that are used as checks (e.g., git rev-parse --verify)
    pub fn run_as_check(self) -> Result<bool> {
        let mut cmd = Command::new(self.command);
        if let Some(dir) = self.workdir {
            cmd.current_dir(dir);
        }
        let output = cmd.args(&self.args).output().with_context(|| {
            format!(
                "Failed to execute command: {} {}",
                self.command,
                self.args.join(" ")
            )
        })?;

        Ok(output.status.success())
    }
}

/// Helper to create a shell command that runs in a shell
pub fn shell_command(command: &str, workdir: &Path) -> Result<()> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command).current_dir(workdir);

    let status = cmd
        .status()
        .with_context(|| format!("Failed to execute shell command: {}", command))?;

    if !status.success() {
        return Err(anyhow!(
            "Shell command failed with exit code {}: {}",
            status.code().unwrap_or(-1),
            command
        ));
    }
    Ok(())
}
