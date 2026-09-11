use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Reads `$EDITOR`. No fallback to `vi`/`nano` — a silent default could
/// surprise a user who expected their configured editor.
pub fn resolve_editor() -> Result<String> {
    std::env::var("EDITOR")
        .ok()
        .filter(|e| !e.trim().is_empty())
        .context("$EDITOR is not set — `dev` needs it for this")
}

/// `sh -c '"$EDITOR" invocation "$1"'` so `$EDITOR` values containing flags
/// (e.g. `"code --wait"`) still work; "dev-editor" fills the required `$0`
/// slot. Shared by `edit_template` (a throwaway buffer) and `open_in_editor`
/// (real files) — both just need the editor to run against some path and
/// block until it exits.
fn spawn_editor(editor: &str, path: &Path) -> Result<()> {
    let path = path.to_str().context("path must be valid UTF-8")?;
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("dev-editor")
        .arg(path)
        .status()
        .with_context(|| format!("failed to launch editor `{editor}`"))?;

    if !status.success() {
        bail!("editor `{editor}` exited with a non-zero status");
    }
    Ok(())
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

    spawn_editor(editor, &path)?;

    std::fs::read_to_string(&path).context("failed to read back commit message")
}

/// Open `editor` directly on `path` (e.g. the repo root, so the user can
/// navigate to and resolve whichever files have conflict markers themselves)
/// and wait for it to exit. Used by the conflict-resolution flow, where the
/// user edits real files in place rather than a throwaway buffer that gets
/// read back.
pub fn open_in_editor(editor: &str, path: &Path) -> Result<()> {
    spawn_editor(editor, path)
}
