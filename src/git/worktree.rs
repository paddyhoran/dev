use super::Git;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// `git worktree add -b <branch> <path> <start_point>`.
pub fn add(git: &Git, branch: &str, path: &Path, start_point: &str) -> Result<()> {
    let path = path.to_str().context("worktree path must be valid UTF-8")?;
    git.run(&["worktree", "add", "-b", branch, path, start_point])?;
    Ok(())
}

/// A feature branch: any git worktree registered under
/// `<repo-root>/.worktrees/<name>`. No separate bookkeeping file — this is
/// the registry `sync` and `done` both discover from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature {
    pub branch: String,
    pub path: PathBuf,
}

/// List feature worktrees by parsing `git worktree list --porcelain` and
/// keeping only entries under `<repo_root>/.worktrees/`.
pub fn list_features(git: &Git, repo_root: &Path) -> Result<Vec<Feature>> {
    let output = git.run(&["worktree", "list", "--porcelain"])?;
    Ok(parse_porcelain(&output, repo_root))
}

/// `git worktree list --porcelain` emits blank-line-separated blocks, each
/// starting with `worktree <path>` followed by `HEAD <sha>` and (for a
/// branch checkout, as opposed to bare/detached) `branch refs/heads/<name>`.
fn parse_porcelain(output: &str, repo_root: &Path) -> Vec<Feature> {
    let worktrees_dir = repo_root.join(".worktrees");
    let mut features = Vec::new();
    let mut current_path: Option<PathBuf> = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/")
            && let Some(path) = current_path
                .take()
                .filter(|p| p.starts_with(&worktrees_dir))
        {
            features.push(Feature {
                branch: branch.to_string(),
                path,
            });
        }
    }

    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_worktrees_under_dot_worktrees() {
        let repo_root = Path::new("/repo");
        let output = "\
worktree /repo
HEAD e71ac0625db9f3f2280940ed3d3ffb053ecd6171
branch refs/heads/main

worktree /repo/.worktrees/feature-a
HEAD 1df56bbd23a457b7a1ebf5b5fe1c2aa044f84ca6
branch refs/heads/feature-a

worktree /repo/.worktrees/feature-b
HEAD 2a38713f8b3657c3e4b847cd600a09b279476e84
branch refs/heads/feature-b
";

        let features = parse_porcelain(output, repo_root);
        assert_eq!(
            features,
            vec![
                Feature {
                    branch: "feature-a".to_string(),
                    path: PathBuf::from("/repo/.worktrees/feature-a"),
                },
                Feature {
                    branch: "feature-b".to_string(),
                    path: PathBuf::from("/repo/.worktrees/feature-b"),
                },
            ]
        );
    }

    #[test]
    fn no_feature_worktrees_is_empty() {
        let repo_root = Path::new("/repo");
        let output = "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n";
        assert!(parse_porcelain(output, repo_root).is_empty());
    }
}
