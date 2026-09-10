use super::Git;
use anyhow::{Context, Result};
use std::path::Path;

/// `git worktree add -b <branch> <path> <start_point>`.
pub fn add(git: &Git, branch: &str, path: &Path, start_point: &str) -> Result<()> {
    let path = path.to_str().context("worktree path must be valid UTF-8")?;
    git.run(&["worktree", "add", "-b", branch, path, start_point])?;
    Ok(())
}
