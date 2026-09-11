mod common;

use common::{Fixture, stderr};

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

[version_file]
path = "Cargo.toml"
pattern = '(?m)^version = "([^"]*)"'
"#;

/// A fixture repo with `cog.toml` + `.dev-config.toml` (with a `[version_file]`
/// pointing at `Cargo.toml`) already committed and pushed.
fn bumpable_fixture() -> Fixture {
    let fx = Fixture::new();
    std::fs::write(
        fx.work.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(fx.work.join("cog.toml"), COG_TOML).unwrap();
    fx.write_config(DEV_CONFIG);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-m", "chore: add cog and dev config"]);
    fx.git(&["push"]);
    fx
}

fn remote_tags(fx: &Fixture) -> String {
    let output = std::process::Command::new("git")
        .args(["ls-remote", "--tags", "origin"])
        .current_dir(&fx.work)
        .output()
        .expect("git ls-remote");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn bumps_version_syncs_manifest_and_pushes_tag() {
    let fx = bumpable_fixture();
    std::fs::write(
        fx.work.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nfeature = true\n",
    )
    .unwrap();
    fx.git(&["add", "Cargo.toml"]);
    fx.git(&["commit", "-m", "feat(cli): add a new feature"]);

    let output = fx.dev(&["bump"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let manifest = std::fs::read_to_string(fx.work.join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("version = \"0.1.0\""),
        "expected synced version in manifest:\n{manifest}"
    );

    assert!(
        remote_tags(&fx).contains("refs/tags/0.1.0"),
        "expected tag 0.1.0 to be pushed to origin"
    );
}

#[test]
fn second_bump_with_no_new_commits_is_a_noop() {
    let fx = bumpable_fixture();
    fx.git(&[
        "commit",
        "--allow-empty",
        "-m",
        "feat(cli): add a new feature",
    ]);
    assert!(fx.dev(&["bump"]).status.success());

    let output = fx.dev(&["bump"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
}

#[test]
fn refuses_on_a_feature_branch() {
    let fx = bumpable_fixture();
    fx.git(&["checkout", "-b", "feature-x"]);

    let output = fx.dev(&["bump"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("main branch"));
}

#[test]
fn refuses_with_a_dirty_working_tree() {
    let fx = bumpable_fixture();
    std::fs::write(fx.work.join("untracked.txt"), "oops").unwrap();

    let output = fx.dev(&["bump"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("not clean"));
}
