use super::Git;
use crate::error::DevError;
use anyhow::Result;
use std::path::PathBuf;

/// Handle onto the repo the current working directory belongs to, resolved
/// once at startup so every command operates from the toplevel regardless of
/// which worktree it was invoked from.
pub struct Repo {
    pub git: Git,
}

impl Repo {
    /// Discover the repo containing the current working directory.
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let probe = Git::new(&cwd);
        let root = probe
            .run(&["rev-parse", "--show-toplevel"])
            .map_err(|_| DevError::NotInRepo)?;
        let git = Git::new(PathBuf::from(root));
        Ok(Self { git })
    }

    pub fn current_branch(&self) -> Result<String> {
        self.git.run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    /// Resolve the main branch name via `origin/HEAD`, falling back to
    /// `main` if that symbolic ref isn't set (e.g. a freshly cloned repo
    /// before the first fetch).
    pub fn main_branch(&self) -> Result<String> {
        match self.git.run(&["symbolic-ref", "refs/remotes/origin/HEAD"]) {
            Ok(reference) => Ok(reference.rsplit('/').next().unwrap_or("main").to_string()),
            Err(_) => Ok("main".to_string()),
        }
    }

    pub fn is_on_main(&self) -> Result<bool> {
        Ok(self.current_branch()? == self.main_branch()?)
    }
}
