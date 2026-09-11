mod common;

use common::{Fixture, ScriptedPrompter};

const CONFIG: &str = r#"
[commit]
types = ["feat", "fix"]
scopes = ["cli", "config"]

[ticket]
footer_template = "Fixes: #{number}"
"#;

fn log_messages(fx: &Fixture) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%B%x00"])
        .current_dir(&fx.work)
        .output()
        .expect("git log");
    String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn blank_issue_number_omits_footer() {
    let fx = Fixture::new();
    fx.write_config(CONFIG);
    let repo = fx.repo();

    let editor = fx.write_fake_editor(
        r#"cat > "$1" <<'BODY'
feat(cli): add a thing

Some body text.
BODY
"#,
    );
    fx.git(&["add", "-A"]);

    let prompter = ScriptedPrompter::new(None, &["feat", "cli"]);
    dev::commands::commit::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("commit should succeed");

    let messages = log_messages(&fx);
    assert_eq!(messages[0], "feat(cli): add a thing\n\nSome body text.");
}

#[test]
fn issue_number_appends_rendered_footer() {
    let fx = Fixture::new();
    fx.write_config(CONFIG);
    let repo = fx.repo();

    let editor = fx.write_fake_editor(
        r#"cat > "$1" <<'BODY'
fix(config): correct a thing
BODY
"#,
    );
    fx.git(&["add", "-A"]);

    let prompter = ScriptedPrompter::new(Some(42), &["fix", "config"]);
    dev::commands::commit::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("commit should succeed");

    let messages = log_messages(&fx);
    assert_eq!(messages[0], "fix(config): correct a thing\n\nFixes: #42");
}

#[test]
fn unedited_subject_aborts_without_committing() {
    let fx = Fixture::new();
    fx.write_config(CONFIG);
    let repo = fx.repo();

    // Leaves the template untouched — subject after the prefix is empty.
    let editor = fx.write_fake_editor(": # no-op, don't touch the file");

    let before = log_messages(&fx).len();
    let prompter = ScriptedPrompter::new(None, &["feat", "cli"]);
    let result = dev::commands::commit::run_with_editor(&repo, &prompter, editor.to_str().unwrap());

    assert!(result.is_err());
    assert_eq!(log_messages(&fx).len(), before, "no commit should be made");
}

#[test]
fn pickers_are_only_offered_configured_values() {
    let fx = Fixture::new();
    fx.write_config(CONFIG);
    let repo = fx.repo();

    let editor = fx.write_fake_editor(
        r#"cat > "$1" <<'BODY'
feat(cli): add a thing
BODY
"#,
    );
    fx.git(&["add", "-A"]);

    let prompter = ScriptedPrompter::new(None, &["feat", "cli"]);
    dev::commands::commit::run_with_editor(&repo, &prompter, editor.to_str().unwrap())
        .expect("commit should succeed");

    let seen = prompter.seen.borrow();
    assert_eq!(
        seen[0],
        (
            "Type".to_string(),
            vec!["feat".to_string(), "fix".to_string()]
        )
    );
    assert_eq!(
        seen[1],
        (
            "Scope".to_string(),
            vec!["cli".to_string(), "config".to_string()]
        )
    );
}
