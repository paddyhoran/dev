use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// `cog bump --auto`, with stdio inherited so Cocogitto's own output (and
/// any prompts from its hooks) reaches the user directly.
pub fn bump_auto(repo_root: &Path) -> Result<()> {
    let status = Command::new("cog")
        .args(["bump", "--auto"])
        .current_dir(repo_root)
        .status()
        .context("failed to run `cog` — is Cocogitto installed and on PATH?")?;

    if !status.success() {
        bail!("`cog bump --auto` failed");
    }
    Ok(())
}
