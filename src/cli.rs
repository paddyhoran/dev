use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "dev",
    version,
    about = "Personal agentic development workflow manager."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new feature branch and worktree
    #[command(alias = "n")]
    New {
        /// Name of the new feature branch
        name: String,
    },

    /// Create a conventional commit interactively
    #[command(alias = "c")]
    Commit,

    /// Pull feature branch commits into main (main branch only)
    #[command(alias = "s")]
    Sync,

    /// Rebase the current feature branch onto latest main
    #[command(alias = "u")]
    Update,

    /// Bump version, update changelog, tag, and push
    #[command(alias = "b")]
    Bump,

    /// Finish and remove a fully-merged feature branch
    #[command(alias = "d")]
    Done,
}
