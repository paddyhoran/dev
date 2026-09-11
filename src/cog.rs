use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// `cog bump --auto`, with stdio inherited so Cocogitto's own output (and
/// any prompts from its hooks) reaches the user directly.
///
/// Note: Cocogitto has its own untracked/uncommitted check, separate from
/// `Repo::is_fully_clean`, and it doesn't know about `dev`'s `.worktrees/`
/// directory. `--skip-untracked` looks like the fix but isn't — it makes cog
/// try to include untracked paths in its bump commit, and it chokes with a
/// libgit2 "invalid path" error on a nested worktree directory specifically.
/// The actual fix is structural: `.worktrees/` must be gitignored in any
/// repo using `dev` (see this project's own `.gitignore`), so it's never
/// untracked in the first place and neither check ever has to reason about
/// it.
pub fn bump_auto(repo_root: &Path) -> Result<()> {
    let status = Command::new("cog")
        .args(["bump", "--auto"])
        .current_dir(repo_root)
        .status()
        .context("failed to run `cog` — is Cocogitto installed and on PATH?")?;

    if !status.success() {
        bail!("`cog bump --auto` failed");
    }
    Ok(())
}
