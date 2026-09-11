/// One commit from `git cherry -v <main> <branch>` that hasn't been merged
/// into `main` yet (patch-content comparison, not SHA — see the module-level
/// note in `commands::sync`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmergedCommit {
    pub sha: String,
    pub subject: String,
}

/// Parse `git cherry -v` output, keeping only `+`-prefixed lines (not yet
/// merged by patch-id — `-`-prefixed lines are already merged and are
/// dropped). Order is preserved, which is oldest-first, matching `cherry`'s
/// own output order.
pub fn parse_unmerged(output: &str) -> Vec<UnmergedCommit> {
    output
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("+ ")?;
            let (sha, subject) = rest.split_once(' ')?;
            Some(UnmergedCommit {
                sha: sha.to_string(),
                subject: subject.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_unmerged_commits_in_order() {
        let output = "\
- 1965a67171d13fad82c1bb456130e77beb69b975 feat(cli): already merged
+ 1df56bbd23a457b7a1ebf5b5fe1c2aa044f84ca6 feat(cli): a second
+ 2a38713f8b3657c3e4b847cd600a09b279476e84 feat(cli): a third";

        let commits = parse_unmerged(output);
        assert_eq!(
            commits,
            vec![
                UnmergedCommit {
                    sha: "1df56bbd23a457b7a1ebf5b5fe1c2aa044f84ca6".to_string(),
                    subject: "feat(cli): a second".to_string(),
                },
                UnmergedCommit {
                    sha: "2a38713f8b3657c3e4b847cd600a09b279476e84".to_string(),
                    subject: "feat(cli): a third".to_string(),
                },
            ]
        );
    }

    #[test]
    fn empty_output_means_nothing_unmerged() {
        assert!(parse_unmerged("").is_empty());
    }

    #[test]
    fn all_merged_means_nothing_unmerged() {
        assert!(parse_unmerged("- abc123 feat(cli): already merged").is_empty());
    }
}
