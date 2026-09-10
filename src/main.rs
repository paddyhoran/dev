mod cli;
mod commands;
mod error;
mod git;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::New { name } => commands::new::run(&name),
        Command::Commit => commands::commit::run(),
        Command::Sync => commands::sync::run(),
        Command::Update => commands::update::run(),
        Command::Bump => commands::bump::run(),
        Command::Done => commands::done::run(),
    }
}
