use crate::git::{Repo, worktree};
use anyhow::{Result, bail};

pub fn run(name: &str) -> Result<()> {
    let repo = Repo::discover()?;

    if !repo.is_main_worktree()? {
        bail!("`dev new` must be run from the main worktree, not a feature worktree");
    }

    let main_branch = repo.main_branch()?;
    repo.git.run(&["fetch", "origin", &main_branch])?;

    if repo.git.status_ok(&[
        "show-ref",
        "--verify",
        "--quiet",
        &format!("refs/heads/{name}"),
    ])? {
        bail!("branch `{name}` already exists");
    }

    let worktree_path = repo.root.join(".worktrees").join(name);
    if worktree_path.exists() {
        bail!("worktree path `{}` already exists", worktree_path.display());
    }

    let start_point = format!("origin/{main_branch}");
    worktree::add(&repo.git, name, &worktree_path, &start_point)?;

    println!("{}", worktree_path.display());
    Ok(())
}
