mod common;

use common::{Fixture, ScriptedPrompter};

fn feature_log(path: &std::path::Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%s"])
        .current_dir(path)
        .output()
        .expect("git log");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn remote_branch_sha(fx: &Fixture, branch: &str) -> String {
    let output = std::process::Command::new("git")
        .args(["ls-remote", "origin", &format!("refs/heads/{branch}")])
        .current_dir(&fx.work)
        .output()
        .expect("git ls-remote");
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn local_sha(path: &std::path::Path) -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .expect("git rev-parse");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn rebases_onto_latest_main_and_force_pushes_only_the_feature_branch() {
    let fx = Fixture::new();
    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("a.txt"), "a\n").unwrap();
    fx.git_in(&a, &["add", "a.txt"]);
    fx.git_in(&a, &["commit", "-m", "feat(cli): add a"]);

    // Advance main independently so there's something to rebase onto.
    std::fs::write(fx.work.join("other.txt"), "main progress\n").unwrap();
    fx.git(&["add", "other.txt"]);
    fx.git(&["commit", "-m", "chore: unrelated main progress"]);
    fx.git(&["push"]);

    let main_sha_before = local_sha(&fx.work);

    let repo = dev::git::Repo::discover_from(&a).expect("discover feature worktree repo");
    let prompter = ScriptedPrompter::for_sync(vec![], vec![]);
    dev::commands::update::run_with_editor(&repo, &prompter, "true")
        .expect("update should succeed");

    let log = feature_log(&a);
    assert!(log.contains(&"feat(cli): add a".to_string()));
    assert!(log.contains(&"chore: unrelated main progress".to_string()));

    assert_eq!(
        remote_branch_sha(&fx, "feature-a"),
        local_sha(&a),
        "origin/feature-a should be force-updated to the rebased HEAD"
    );
    assert_eq!(
        remote_branch_sha(&fx, "main"),
        main_sha_before,
        "origin/main must be untouched by `dev update`"
    );
}

#[test]
fn refuses_on_main() {
    let fx = Fixture::new();
    let repo = fx.repo();
    let prompter = ScriptedPrompter::for_sync(vec![], vec![]);

    let result = dev::commands::update::run_with_editor(&repo, &prompter, "true");
    assert!(result.is_err());
    assert!(format!("{:#}", result.unwrap_err()).contains("feature branch"));
}

fn diverge_main_and_feature_on_readme(fx: &Fixture) -> std::path::PathBuf {
    let a = fx.add_feature_worktree("feature-a");
    std::fs::write(a.join("README.md"), "feature change\n").unwrap();
    fx.git_in(&a, &["commit", "-am", "feat(cli): conflicting change"]);

    std::fs::write(fx.work.join("README.md"), "main change\n").unwrap();
    fx.git(&["commit", "-am", "chore: unrelated main change"]);
    fx.git(&["push"]);
    a
}

#[test]
fn resolving_the_conflict_via_editor_continues_the_rebase_and_pushes() {
    let fx = Fixture::new();
    let a = diverge_main_and_feature_on_readme(&fx);

    let editor = fx.write_fake_editor("printf 'resolved\\n' > \"$1/README.md\"");

    let repo = dev::git::Repo::discover_from(&a).expect("discover feature worktree repo");
    let prompter = ScriptedPrompter::for_sync(vec![], vec![]);
    dev::commands::update::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("update should succeed once the conflict is resolved");

    assert_eq!(
        std::fs::read_to_string(a.join("README.md")).unwrap(),
        "resolved\n"
    );
    assert!(feature_log(&a).contains(&"feat(cli): conflicting change".to_string()));
    assert_eq!(remote_branch_sha(&fx, "feature-a"), local_sha(&a));
}

#[test]
fn declining_to_retry_an_unresolved_conflict_aborts_the_rebase_without_pushing() {
    let fx = Fixture::new();
    let a = diverge_main_and_feature_on_readme(&fx);
    let original_remote_sha = remote_branch_sha(&fx, "feature-a");

    // Does nothing — conflict markers are left exactly as rebase wrote them.
    let editor = fx.write_fake_editor("true");

    let repo = dev::git::Repo::discover_from(&a).expect("discover feature worktree repo");
    let prompter = ScriptedPrompter::for_sync(vec![], vec![false]);
    dev::commands::update::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("update should stop cleanly, not error, on a declined retry");

    let status = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", "README.md"])
        .current_dir(&a)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&status.stdout).trim().is_empty(),
        "rebase --abort should leave README.md clean"
    );
    assert_eq!(
        remote_branch_sha(&fx, "feature-a"),
        original_remote_sha,
        "an aborted rebase must not be pushed"
    );
}
