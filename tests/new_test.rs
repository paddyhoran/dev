mod common;

use common::{Fixture, stderr, stdout};
use std::process::Command;

#[test]
fn creates_branch_and_worktree() {
    let fx = Fixture::new();

    let output = fx.dev(&["new", "my-feature"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let expected = fx.work.join(".worktrees").join("my-feature");
    let printed = std::path::PathBuf::from(stdout(&output));
    assert_eq!(
        std::fs::canonicalize(&printed).unwrap(),
        std::fs::canonicalize(&expected).unwrap()
    );
    assert!(expected.join("README.md").exists());

    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&expected)
        .output()
        .unwrap();
    assert_eq!(stdout(&branch), "my-feature");
}

#[test]
fn refuses_duplicate_branch_name() {
    let fx = Fixture::new();
    assert!(fx.dev(&["new", "dup"]).status.success());

    let output = fx.dev(&["new", "dup"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn refuses_from_inside_a_feature_worktree() {
    let fx = Fixture::new();
    let first = fx.dev(&["new", "first"]);
    assert!(first.status.success());
    let feature_path = std::path::PathBuf::from(stdout(&first));

    let output = fx.dev_in(&feature_path, &["new", "second"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("main worktree"));
}
