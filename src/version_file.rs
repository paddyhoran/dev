use crate::config::VersionFileConfig;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

/// Rewrite the version captured by `config.pattern`'s first capture group in
/// `config.path` to `tag`. A leading `v`/`V` is stripped from `tag` first —
/// git tags commonly carry one (e.g. `v1.2.3`) but manifest version fields
/// generally can't (`cargo` rejects `version = "v1.2.3"` outright).
pub fn sync(repo_root: &Path, config: &VersionFileConfig, tag: &str) -> Result<()> {
    let version = tag.trim_start_matches(['v', 'V']);

    let path = repo_root.join(&config.path);
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read version file {}", path.display()))?;

    let re = Regex::new(&config.pattern)
        .with_context(|| format!("invalid [version_file] pattern: {}", config.pattern))?;
    let captures = re.captures(&contents).with_context(|| {
        format!(
            "[version_file] pattern did not match anything in {}",
            path.display()
        )
    })?;
    let group = captures.get(1).with_context(|| {
        "[version_file] pattern must have one capture group around the version".to_string()
    })?;

    let mut updated = contents.clone();
    updated.replace_range(group.range(), version);

    std::fs::write(&path, updated)
        .with_context(|| format!("failed to write version file {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_captured_version_and_strips_v_prefix() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let config = VersionFileConfig {
            path: "Cargo.toml".to_string(),
            pattern: r#"(?m)^version = "([^"]*)""#.to_string(),
        };
        sync(dir.path(), &config, "v1.2.3").unwrap();

        let updated = std::fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert_eq!(updated, "[package]\nname = \"demo\"\nversion = \"1.2.3\"\n");
    }

    #[test]
    fn errors_when_pattern_does_not_match() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();

        let config = VersionFileConfig {
            path: "Cargo.toml".to_string(),
            pattern: r#"(?m)^version = "([^"]*)""#.to_string(),
        };
        assert!(sync(dir.path(), &config, "1.2.3").is_err());
    }
}
