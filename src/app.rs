use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum PoolCommands {
    /// List existing pools.
    List,
    /// Show the contents of the current pool.
    Show,
    /// Create a new pool.
    New {
        label: String,
        #[clap(parse(from_os_str))]
        root: Option<PathBuf>,
    },
    /// Set the current pool.
    Set { label: String },
    /// Move the current pool root.
    ///
    /// This command only updates the pool configuration file, it does
    /// not however move the managed repositories.
    Move {
        #[clap(parse(from_os_str))]
        root: PathBuf,
    },
    /// Change the current pool label.
    Rename { label: String },
    /// Add an exclusion rule for this pool discovery.
    Exclude { pattern: String },
    /// Clone missing repositories of the current pool, checkout appropriate branches.
    Checkout,
    /// Save managed repositories current state.
    Discover,
}

#[derive(Debug, Subcommand)]
pub enum GitCommand {
    #[clap(external_subcommand)]
    Command(Vec<String>),
}

#[derive(Debug, Subcommand)]
pub enum ApplicationCommands {
    /// Create or manage a pool.
    ///
    /// When run without a subcommand, `gmux pool` will output
    /// the current pool.
    Pool {
        #[clap(subcommand)]
        pool_command: Option<PoolCommands>,
    },
    /// Run any git command on every repository of the current pool.
    Command {
        #[clap(short, long)]
        exclude_filter: Option<String>,
        #[clap(short, long)]
        filter: Option<String>,
        #[clap(subcommand)]
        command: GitCommand,
    },
}

#[derive(Debug, Parser)]
#[clap(author, version, about, name = "gmux")]
/// Manage multiple git repositories with ease.
pub struct Application {
    #[clap(subcommand)]
    pub command: ApplicationCommands,
}
