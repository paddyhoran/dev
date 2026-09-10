use super::Git;
use crate::error::DevError;
use anyhow::Result;
use std::path::PathBuf;

/// Handle onto the repo the current working directory belongs to, resolved
/// once at startup so every command operates from the toplevel regardless of
/// which worktree it was invoked from.
pub struct Repo {
    pub root: PathBuf,
    pub git: Git,
}

impl Repo {
    /// Discover the repo containing the current working directory. `root` is
    /// the toplevel of whichever worktree the command was invoked from — not
    /// necessarily the main worktree, see `is_main_worktree`.
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let probe = Git::new(&cwd);
        let root = probe
            .run(&["rev-parse", "--show-toplevel"])
            .map_err(|_| DevError::NotInRepo)?;
        let root = PathBuf::from(root);
        let git = Git::new(&root);
        Ok(Self { root, git })
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

    /// A linked worktree has its own `--git-dir` (`<common>/worktrees/<name>`)
    /// distinct from `--git-common-dir` (the shared `.git`); the main
    /// worktree is the one where they're equal.
    pub fn is_main_worktree(&self) -> Result<bool> {
        let git_dir = self
            .git
            .run(&["rev-parse", "--path-format=absolute", "--git-dir"])?;
        let common_dir =
            self.git
                .run(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
        Ok(git_dir == common_dir)
    }
}
