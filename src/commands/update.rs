use crate::editor;
use crate::error::DevError;
use crate::git::Repo;
use crate::git::conflict::{self, Resolution};
use crate::picker::Prompter;
use anyhow::Result;

pub fn run(repo: &Repo, prompter: &dyn Prompter) -> Result<()> {
    let editor_bin = editor::resolve_editor()?;
    run_with_editor(repo, prompter, &editor_bin)
}

/// The real flow, taking the editor binary as a parameter — same
/// `run`/`run_with_editor` split as `commands::commit` and `commands::sync`,
/// for the same reason (tests can't drive a real interactive editor
/// headlessly, and mutating `$EDITOR` globally would race across parallel
/// tests).
pub fn run_with_editor(repo: &Repo, prompter: &dyn Prompter, editor_bin: &str) -> Result<()> {
    if repo.is_on_main()? {
        return Err(DevError::NotOnFeatureBranch {
            command: "update",
            branch: repo.current_branch()?,
        }
        .into());
    }

    let branch = repo.current_branch()?;
    let main = repo.main_branch()?;

    // Fetch everything, not just main: `--force-with-lease` below needs a
    // fresh `origin/<branch>` to safely detect if someone else pushed to
    // this feature branch since we last looked.
    repo.git.run(&["fetch", "origin"])?;

    if repo
        .git
        .run(&["rebase", &format!("origin/{main}")])
        .is_err()
        && conflict::resolve_or_abort(repo, prompter, editor_bin, "rebase")? == Resolution::Aborted
    {
        println!("dev update: stopping after an aborted rebase");
        return Ok(());
    }

    // Explicit refspec, not a bare `--force-with-lease`: `dev new` sets a
    // new feature branch's upstream to `origin/<main>` (its start point),
    // not `origin/<branch>` — relying on that tracking here could push to
    // the wrong ref entirely.
    repo.git.run(&[
        "push",
        "--force-with-lease",
        "origin",
        &format!("{branch}:{branch}"),
    ])?;

    Ok(())
}
