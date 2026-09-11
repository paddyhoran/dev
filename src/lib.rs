pub mod cli;
pub mod cog;
pub mod commands;
pub mod commit_template;
pub mod config;
pub mod editor;
pub mod error;
pub mod git;
pub mod picker;
pub mod version_file;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use git::Repo;
use picker::DialoguerPrompter;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let repo = Repo::discover_from(&std::env::current_dir()?)?;

    match cli.command {
        Command::New { name } => commands::new::run(&repo, &name),
        Command::Commit => commands::commit::run(&repo, &DialoguerPrompter),
        Command::Sync => commands::sync::run(&repo, &DialoguerPrompter),
        Command::Update => commands::update::run(&repo, &DialoguerPrompter),
        Command::Bump => commands::bump::run(&repo),
        Command::Done => commands::done::run(&repo),
    }
}
