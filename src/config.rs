use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::Path;

/// `.dev-config.toml`, read from the repo root. Supplies the option lists
/// `dev commit`'s pickers offer and the ticket-footer template.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub commit: CommitConfig,
    #[serde(default)]
    pub ticket: TicketConfig,
    pub version_file: Option<VersionFileConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CommitConfig {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TicketConfig {
    #[serde(default = "default_footer_template")]
    pub footer_template: String,
}

impl Default for TicketConfig {
    fn default() -> Self {
        Self {
            footer_template: default_footer_template(),
        }
    }
}

fn default_footer_template() -> String {
    "Fixes: #{number}".to_string()
}

/// Where `dev bump` should write the post-bump version, for manifests cog
/// itself doesn't know how to update (it only manages the changelog, tag,
/// and its own bump commit). `pattern` is a regex with exactly one capture
/// group wrapping the version text to replace — language-agnostic by
/// design, so it works for Cargo.toml, package.json, a bare VERSION file,
/// or anything else, at the cost of the user anchoring the pattern
/// precisely enough not to match an unrelated `version = "..."` elsewhere
/// in the file (e.g. a pinned dependency).
#[derive(Debug, Deserialize)]
pub struct VersionFileConfig {
    pub path: String,
    pub pattern: String,
}

impl Config {
    pub fn load(repo_root: &Path) -> Result<Self> {
        let path = repo_root.join(".dev-config.toml");
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("missing config file: {}", path.display()))?;
        let config: Config =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;

        if config.commit.types.is_empty() {
            bail!("{}: [commit] types must not be empty", path.display());
        }
        if config.commit.scopes.is_empty() {
            bail!("{}: [commit] scopes must not be empty", path.display());
        }

        Ok(config)
    }

    pub fn render_ticket_footer(&self, issue_number: u64) -> String {
        self.ticket
            .footer_template
            .replace("{number}", &issue_number.to_string())
    }
}
