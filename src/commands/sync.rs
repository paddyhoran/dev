use crate::editor;
use crate::error::DevError;
use crate::git::conflict::{self, Resolution};
use crate::git::{Repo, cherry, worktree};
use crate::picker::Prompter;
use anyhow::Result;

/// A feature branch's oldest commit not yet merged into main — the one
/// `sync` would apply next for that branch.
struct Candidate {
    branch: String,
    sha: String,
    subject: String,
}

impl Candidate {
    fn label(&self) -> String {
        format!("{}: {} {}", self.branch, &self.sha[..7], self.subject)
    }
}

pub fn run(repo: &Repo, prompter: &dyn Prompter) -> Result<()> {
    // Resolved eagerly, even though it's only needed on the conflict path —
    // matches `commit`'s "you need $EDITOR configured to use this tool at
    // all" posture, and keeps `run_with_editor` fully parameterized for
    // tests without a lazy-resolution seam to also fake out.
    let editor_bin = editor::resolve_editor()?;
    run_with_editor(repo, prompter, &editor_bin)
}

/// The real flow, taking the editor binary as a parameter rather than
/// reading `$EDITOR` itself — see `commands::commit::run_with_editor` for
/// the same pattern and why (tests can't drive a real interactive editor
/// headlessly, and mutating `$EDITOR` globally would race across parallel
/// tests).
pub fn run_with_editor(repo: &Repo, prompter: &dyn Prompter, editor_bin: &str) -> Result<()> {
    if !repo.is_on_main()? {
        return Err(DevError::NotOnMainBranch {
            command: "sync",
            branch: repo.current_branch()?,
        }
        .into());
    }

    repo.git.run(&["fetch", "--all"])?;

    let mut synced_any = false;

    'outer: loop {
        let candidates = next_candidates(repo)?;
        if candidates.is_empty() {
            println!("dev sync: nothing to sync");
            break;
        }

        let labels: Vec<String> = candidates.iter().map(Candidate::label).collect();
        let chosen =
            prompter.select_many("Commits to sync (select none to stop for now)", &labels)?;
        if chosen.is_empty() {
            break;
        }

        for idx in chosen {
            let candidate = &candidates[idx];
            if repo
                .git
                .run(&["cherry-pick", candidate.sha.as_str()])
                .is_err()
            {
                println!(
                    "cherry-pick of `{}` from `{}` conflicted",
                    &candidate.sha[..7],
                    candidate.branch
                );
                if conflict::resolve_or_abort(repo, prompter, editor_bin, "cherry-pick")?
                    == Resolution::Aborted
                {
                    println!("dev sync: stopping after an aborted cherry-pick");
                    break 'outer;
                }
            }
            repo.git.run(&["push"])?;
            synced_any = true;
        }
    }

    if synced_any && prompter.confirm("Run `dev bump` now?")? {
        super::bump::run(repo)?;
    }

    Ok(())
}

/// One candidate per feature branch: its oldest commit not yet merged into
/// main, by patch content (`git cherry -v`) rather than SHA — see
/// `git::cherry` for why that's the right comparison.
fn next_candidates(repo: &Repo) -> Result<Vec<Candidate>> {
    let main = repo.main_branch()?;
    let features = worktree::list_features(&repo.git, &repo.root)?;

    let mut candidates = Vec::new();
    for feature in features {
        let output = repo.git.run(&["cherry", "-v", &main, &feature.branch])?;
        if let Some(oldest) = cherry::parse_unmerged(&output).into_iter().next() {
            candidates.push(Candidate {
                branch: feature.branch,
                sha: oldest.sha,
                subject: oldest.subject,
            });
        }
    }
    Ok(candidates)
}
