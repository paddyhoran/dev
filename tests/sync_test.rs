mod common;

use common::{Fixture, ScriptedPrompter};

const COG_TOML: &str = r#"
skip_untracked = false
[git_hooks]
[commit_types]
[changelog]
path = "CHANGELOG.md"
authors = []
[bump_profiles]
"#;

const DEV_CONFIG: &str = r#"
[commit]
types = ["feat", "fix", "chore"]
scopes = ["cli"]
"#;

fn main_log(fx: &Fixture) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%s"])
        .current_dir(&fx.work)
        .output()
        .expect("git log");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn origin_log(fx: &Fixture) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["log", "origin/main", "--format=%s"])
        .current_dir(&fx.work)
        .output()
        .expect("git log origin/main");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn nothing_to_sync_with_no_feature_branches() {
    let fx = Fixture::new();
    let repo = fx.repo();
    let prompter = ScriptedPrompter::for_sync(vec![], vec![]);

    dev::commands::sync::run_with_editor(&repo, &prompter, "true").expect("sync should succeed");
}

#[test]
fn syncs_one_commit_from_each_of_two_branches_and_pushes() {
    let fx = Fixture::new();

    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("a.txt"), "a\n").unwrap();
    fx.git_in(&a, &["add", "a.txt"]);
    fx.git_in(&a, &["commit", "-m", "feat(cli): add a"]);

    let b = fx.add_feature_worktree("feature-b");
    std::fs::write(b.join("b.txt"), "b\n").unwrap();
    fx.git_in(&b, &["add", "b.txt"]);
    fx.git_in(&b, &["commit", "-m", "feat(cli): add b"]);

    let repo = fx.repo();
    // Round 1: both candidates offered, select both. Round 2: nothing left.
    let prompter = ScriptedPrompter::for_sync(vec![vec![0, 1]], vec![false]);

    dev::commands::sync::run_with_editor(&repo, &prompter, "true").expect("sync should succeed");

    let log = main_log(&fx);
    assert!(log.contains(&"feat(cli): add a".to_string()));
    assert!(log.contains(&"feat(cli): add b".to_string()));

    let origin = origin_log(&fx);
    assert!(origin.contains(&"feat(cli): add a".to_string()));
    assert!(origin.contains(&"feat(cli): add b".to_string()));
}

#[test]
fn applies_multiple_commits_on_one_branch_oldest_first_across_rounds() {
    let fx = Fixture::new();

    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("a.txt"), "1\n").unwrap();
    fx.git_in(&a, &["add", "a.txt"]);
    fx.git_in(&a, &["commit", "-m", "feat(cli): first"]);
    std::fs::write(a.join("a.txt"), "1\n2\n").unwrap();
    fx.git_in(&a, &["add", "a.txt"]);
    fx.git_in(&a, &["commit", "-m", "feat(cli): second"]);

    let repo = fx.repo();
    // Round 1: only "first" is a candidate (oldest unmerged). Round 2, after
    // recompute: "second" becomes the candidate. Round 3: nothing left.
    let prompter = ScriptedPrompter::for_sync(vec![vec![0], vec![0]], vec![false]);

    dev::commands::sync::run_with_editor(&repo, &prompter, "true").expect("sync should succeed");

    let log = main_log(&fx);
    let first_pos = log.iter().position(|s| s == "feat(cli): first").unwrap();
    let second_pos = log.iter().position(|s| s == "feat(cli): second").unwrap();
    assert!(
        second_pos < first_pos,
        "git log is newest-first, so \"second\" (applied later) must come before \"first\": {log:?}"
    );
}

#[test]
fn confirming_bump_runs_it_and_pushes_a_tag() {
    let fx = Fixture::new();
    std::fs::write(fx.work.join("cog.toml"), COG_TOML).unwrap();
    fx.write_config(DEV_CONFIG);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-m", "chore: add cog and dev config"]);
    fx.git(&["push"]);

    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("a.txt"), "a\n").unwrap();
    fx.git_in(&a, &["add", "a.txt"]);
    fx.git_in(&a, &["commit", "-m", "feat(cli): add a"]);

    let repo = fx.repo();
    let prompter = ScriptedPrompter::for_sync(vec![vec![0]], vec![true]);
    dev::commands::sync::run_with_editor(&repo, &prompter, "true").expect("sync should succeed");

    let output = std::process::Command::new("git")
        .args(["ls-remote", "--tags", "origin"])
        .current_dir(&fx.work)
        .output()
        .unwrap();
    assert!(
        !String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "confirming the bump prompt should have run `dev bump` and pushed a tag"
    );
}

#[test]
fn declining_the_bump_prompt_leaves_no_new_tag() {
    let fx = Fixture::new();
    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("a.txt"), "a\n").unwrap();
    fx.git_in(&a, &["add", "a.txt"]);
    fx.git_in(&a, &["commit", "-m", "feat(cli): add a"]);

    let repo = fx.repo();
    let prompter = ScriptedPrompter::for_sync(vec![vec![0]], vec![false]);
    dev::commands::sync::run_with_editor(&repo, &prompter, "true").expect("sync should succeed");

    let output = std::process::Command::new("git")
        .args(["tag"])
        .current_dir(&fx.work)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "declining the bump prompt should not create a tag"
    );
}

#[test]
fn refuses_when_not_on_main() {
    let fx = Fixture::new();
    fx.git(&["checkout", "-b", "not-main"]);
    let repo = fx.repo();
    let prompter = ScriptedPrompter::for_sync(vec![], vec![]);

    let result = dev::commands::sync::run_with_editor(&repo, &prompter, "true");
    assert!(result.is_err());
    assert!(format!("{:#}", result.unwrap_err()).contains("main branch"));
}

fn diverge_main_and_feature_on_readme(fx: &Fixture) -> std::path::PathBuf {
    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("README.md"), "feature change\n").unwrap();
    fx.git_in(&a, &["commit", "-am", "feat(cli): conflicting change"]);

    // Diverge main so the cherry-pick can't apply cleanly.
    std::fs::write(fx.work.join("README.md"), "main change\n").unwrap();
    fx.git(&["commit", "-am", "chore: unrelated main change"]);
    a
}

/// Status of just `README.md` — the one file involved in the conflict.
/// Scoped rather than whole-tree so the fixture's own `fake_editor.sh`
/// (deliberately untracked, unrelated to conflict cleanliness) doesn't read
/// as a leftover.
fn readme_status(fx: &Fixture) -> String {
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", "README.md"])
        .current_dir(&fx.work)
        .output()
        .unwrap();
    String::from_utf8_lossy(&status.stdout).trim().to_string()
}

#[test]
fn resolving_the_conflict_via_editor_continues_and_pushes() {
    let fx = Fixture::new();
    diverge_main_and_feature_on_readme(&fx);

    // Overwrites the conflicted file with resolved content, standing in for
    // the user editing away the conflict markers themselves.
    let editor = fx.write_fake_editor("printf 'resolved\\n' > \"$1/README.md\"");

    let repo = fx.repo();
    // Round 1 selects the conflicting candidate. Our "resolution" content
    // doesn't reproduce a byte-identical patch to the original feature
    // commit (it's an arbitrary overwrite, not a real 3-way merge result),
    // so `git cherry` still considers it unmerged by patch-id afterwards —
    // an inherent property of content-based tracking for real conflicts,
    // not a bug. Round 2 selects nothing to stop rather than re-attempting
    // it. No "retry?" confirm is ever asked (resolution succeeds in one
    // pass), so the only confirm is the end-of-run bump prompt.
    let prompter = ScriptedPrompter::for_sync(vec![vec![0], vec![]], vec![false]);

    dev::commands::sync::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("sync should succeed once the conflict is resolved");

    assert_eq!(
        std::fs::read_to_string(fx.work.join("README.md")).unwrap(),
        "resolved\n"
    );
    assert!(main_log(&fx).contains(&"feat(cli): conflicting change".to_string()));
    assert!(origin_log(&fx).contains(&"feat(cli): conflicting change".to_string()));
    assert!(readme_status(&fx).is_empty());
}

#[test]
fn declining_to_retry_an_unresolved_conflict_aborts_cleanly() {
    let fx = Fixture::new();
    diverge_main_and_feature_on_readme(&fx);

    // Does nothing — conflict markers are left exactly as cherry-pick wrote them.
    let editor = fx.write_fake_editor("true");

    let repo = fx.repo();
    // Only the "conflicts remain, try again?" confirm is ever reached here
    // (sync stops right after the abort, so no end-of-run bump prompt).
    let prompter = ScriptedPrompter::for_sync(vec![vec![0]], vec![false]);

    dev::commands::sync::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("sync should stop cleanly, not error, on a declined retry");

    assert!(!main_log(&fx).contains(&"feat(cli): conflicting change".to_string()));
    assert!(
        readme_status(&fx).is_empty(),
        "cherry-pick --abort should leave README.md clean"
    );
}
