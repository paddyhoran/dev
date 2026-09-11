use super::Git;
use crate::error::DevError;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Handle onto the repo the current working directory belongs to, resolved
/// once at startup so every command operates from the toplevel regardless of
/// which worktree it was invoked from.
pub struct Repo {
    pub root: PathBuf,
    pub git: Git,
}

impl Repo {
    /// Discover the repo containing `start`.
    ///
    /// Tests can point at a fixture repo directly instead of mutating the
    /// process-wide cwd, which would race across parallel tests. `root` is
    /// the toplevel of whichever worktree `start` is inside of — not
    /// necessarily the main worktree, see `is_main_worktree`.
    pub fn discover_from(start: &Path) -> Result<Self> {
        let probe = Git::new(start);
        let root = probe
            .run(&["rev-parse", "--show-toplevel"])
            .map_err(|_| DevError::NotInRepo)?;
        let root = PathBuf::from(root);
        let git = Git::new(&root);
        Ok(Self { root, git })
    }

    /// Returns the name of the current branch.
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

    /// Whether we are currently on the main branch.
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

    /// Whether the worktree has anything unstaged or untracked. Staged
    /// changes don't count — `dev commit` calls this to make sure nothing
    /// besides what's deliberately staged is about to be swept into the
    /// commit, not to demand a fully clean tree (that's `dev bump`'s job).
    pub fn has_unstaged_or_untracked_changes(&self) -> Result<bool> {
        let status = self.git.run(&["status", "--porcelain"])?;
        Ok(porcelain_has_unstaged_or_untracked(&status))
    }
}

/// A `git status --porcelain` line is `XY <path>`, where `X` is the status
/// relative to the index (staged) and `Y` relative to the worktree
/// (unstaged). Untracked files are reported as `??`. Pulled out of
/// `has_unstaged_or_untracked_changes` so it's unit-testable against sample
/// porcelain output without a real repo.
fn porcelain_has_unstaged_or_untracked(status: &str) -> bool {
    status.lines().any(|line| {
        let code = line.as_bytes();
        code.first() == Some(&b'?') || code.get(1).is_some_and(|&y| y != b' ')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_status_has_no_unstaged_or_untracked() {
        assert!(!porcelain_has_unstaged_or_untracked(""));
    }

    #[test]
    fn staged_only_is_fine() {
        assert!(!porcelain_has_unstaged_or_untracked(
            "M  staged.txt\nA  new.txt"
        ));
    }

    #[test]
    fn unstaged_modification_is_flagged() {
        assert!(porcelain_has_unstaged_or_untracked(" M unstaged.txt"));
    }

    #[test]
    fn untracked_file_is_flagged() {
        assert!(porcelain_has_unstaged_or_untracked("?? new.txt"));
    }

    #[test]
    fn staged_plus_unstaged_on_same_file_is_flagged() {
        // Staged one hunk, then edited further without re-staging.
        assert!(porcelain_has_unstaged_or_untracked("MM both.txt"));
    }
}
