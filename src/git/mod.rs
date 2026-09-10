mod repo;

pub use repo::Repo;

use crate::error::DevError;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Thin wrapper around the `git` binary. Every subprocess call in the
/// codebase should go through here, so command implementations stay
/// testable against a real temp repo rather than mocked git behavior.
pub struct Git {
    cwd: PathBuf,
}

impl Git {
    pub fn new(cwd: impl AsRef<Path>) -> Self {
        Self {
            cwd: cwd.as_ref().to_path_buf(),
        }
    }

    /// Run a git command and capture its stdout (trimmed). Non-zero exit
    /// becomes a `DevError::GitCommandFailed` carrying stderr.
    pub fn run(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.cwd)
            .output()
            .with_context(|| format!("failed to spawn `git {}`", args.join(" ")))?;

        if !output.status.success() {
            return Err(DevError::GitCommandFailed {
                args: args.join(" "),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            }
            .into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
