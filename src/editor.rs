use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::Command;

/// Reads `$EDITOR`. No fallback to `vi`/`nano` — a silent default could
/// surprise a user who expected their configured editor.
pub fn resolve_editor() -> Result<String> {
    std::env::var("EDITOR")
        .ok()
        .filter(|e| !e.trim().is_empty())
        .context("$EDITOR is not set — `dev commit` needs it to write the commit message")
}

/// Write `initial_contents` to a temp file, open it in `editor` (as resolved
/// by `resolve_editor`), and return the raw file contents after the editor
/// exits (comment stripping happens in `commit_template::finalize`, since
/// what counts as a comment is specific to the commit-message template).
/// Takes `editor` as a parameter rather than reading `$EDITOR` itself so
/// tests can supply a fake editor directly, without mutating process-wide
/// env state that would race across parallel tests.
pub fn edit_template(editor: &str, initial_contents: &str) -> Result<String> {
    let mut file = tempfile::Builder::new()
        .suffix(".txt")
        .tempfile()
        .context("failed to create temp file for commit message")?;
    file.write_all(initial_contents.as_bytes())?;
    file.flush()?;
    let path = file.path().to_path_buf();

    // `sh -c '"$EDITOR" invocation "$1"'` so `$EDITOR` values containing
    // flags (e.g. `"code --wait"`) still work; "dev-editor" fills the
    // required $0 slot.
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("dev-editor")
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor `{editor}`"))?;

    if !status.success() {
        bail!("editor `{editor}` exited with a non-zero status");
    }

    std::fs::read_to_string(&path).context("failed to read back commit message")
}
