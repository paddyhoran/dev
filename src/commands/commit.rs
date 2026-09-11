use crate::commit_template;
use crate::config::Config;
use crate::editor;
use crate::git::Repo;
use crate::picker::Prompter;
use anyhow::Result;

pub fn run(repo: &Repo, prompter: &dyn Prompter) -> Result<()> {
    let editor_bin = editor::resolve_editor()?;
    run_with_editor(repo, prompter, &editor_bin)
}

/// The real flow, taking the editor binary as a parameter rather than
/// reading `$EDITOR` itself. Exposed (not just `pub(crate)`) so integration
/// tests can drive it with a fake editor script directly, instead of
/// mutating the process-wide `$EDITOR` env var — which would race across
/// tests running in parallel in the same process.
pub fn run_with_editor(repo: &Repo, prompter: &dyn Prompter, editor_bin: &str) -> Result<()> {
    let config = Config::load(&repo.root)?;

    let issue_number = prompter.ask_issue_number()?;
    let commit_type = prompter.select("Type", &config.commit.types)?;
    let scope = prompter.select("Scope", &config.commit.scopes)?;
    let ticket_footer = issue_number.map(|n| config.render_ticket_footer(n));

    let template = commit_template::render(&commit_type, &scope, ticket_footer.as_deref());
    let edited = editor::edit_template(editor_bin, &template)?;
    let message =
        commit_template::finalize(&commit_type, &scope, &edited, ticket_footer.as_deref())?;

    repo.git.run(&["commit", "-m", &message])?;
    Ok(())
}
