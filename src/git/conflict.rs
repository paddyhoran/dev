use super::Repo;
use crate::editor;
use crate::picker::Prompter;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Continued,
    Aborted,
}

/// Shared flow for a git operation that stopped mid-way with conflicts.
/// `subcommand` is the git porcelain command name used for
/// `--continue`/`--abort` — `"cherry-pick"` today; `dev update`'s rebase
/// reuses this unchanged by passing `"rebase"`. `editor` is the resolved
/// `$EDITOR` binary, taken as a parameter (rather than resolving it here)
/// so tests can supply a fake editor directly, without mutating
/// process-wide env state that would race across parallel tests — same
/// reasoning as `editor::edit_template`.
///
/// Opens `editor` on the repo root (not a template — the user resolves
/// conflict markers directly in their own files), then checks whether any
/// originally-conflicted file still contains conflict markers. If so, asks
/// whether to try again or give up; giving up runs `--abort`. Once markers
/// are gone, stages exactly the originally-conflicted paths (not
/// `git add -A`, which would also sweep in unrelated untracked files) and
/// runs `--continue`.
///
/// The conflicted-paths list is captured once, up front — not by re-running
/// `git diff --diff-filter=U` after the editor closes. That check is
/// index-based (whether the path has been `git add`ed to mark it resolved),
/// not content-based, so it can't change just from editing a file's text;
/// checking it again post-edit would always still report the file as
/// conflicted, since nothing has staged it yet.
pub fn resolve_or_abort(
    repo: &Repo,
    prompter: &dyn Prompter,
    editor: &str,
    subcommand: &str,
) -> Result<Resolution> {
    let conflicted = conflicted_paths(repo)?;
    println!("Conflicting files:");
    for path in &conflicted {
        println!("  {path}");
    }

    loop {
        editor::spawn_editor(editor, &repo.root)?;

        if !any_has_conflict_markers(repo, &conflicted) {
            let mut add_args = vec!["add"];
            add_args.extend(conflicted.iter().map(String::as_str));
            repo.git.run(&add_args)?;
            repo.git.run(&[subcommand, "--continue"])?;
            return Ok(Resolution::Continued);
        }

        if !prompter.confirm("Conflicts remain — open the editor again?")? {
            repo.git.run(&[subcommand, "--abort"])?;
            return Ok(Resolution::Aborted);
        }
    }
}

fn conflicted_paths(repo: &Repo) -> Result<Vec<String>> {
    let output = repo.git.run(&["diff", "--name-only", "--diff-filter=U"])?;
    Ok(output.lines().map(str::to_string).collect())
}

/// Whether any of `paths` (relative to the repo root) still contains git's
/// conflict-marker lines. A file that's been deleted as its own resolution
/// (a valid outcome) reads as having no markers.
fn any_has_conflict_markers(repo: &Repo, paths: &[String]) -> bool {
    paths.iter().any(|path| {
        std::fs::read_to_string(repo.root.join(path))
            .map(|contents| {
                contents
                    .lines()
                    .any(|line| line.starts_with("<<<<<<<") || line.starts_with(">>>>>>>"))
            })
            .unwrap_or(false)
    })
}
