use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum PoolCommands {
    /// List existing pools.
    List,
    /// Create a new pool.
    New {
        label: String,
        #[structopt(parse(from_os_str))]
        root: Option<PathBuf>,
    },
    /// Set the current pool.
    Set { label: String },
    /// Move the current pool root.
    ///
    /// This command only updates the pool configuration file, it does
    /// not however move the managed repositories.
    Move {
        #[structopt(parse(from_os_str))]
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

#[derive(Debug, StructOpt)]
pub struct PoolSubcommand {
    #[structopt(subcommand)]
    pub command: Option<PoolCommands>,
}

#[derive(Debug, StructOpt)]
pub enum GitCommand {
    #[structopt(external_subcommand)]
    Command(Vec<String>),
}

#[derive(Debug, StructOpt)]
pub enum ApplicationCommands {
    /// Create or manage a pool.
    ///
    /// When run without a subcommand, `rit pool` will output
    /// the current pool.
    Pool(PoolSubcommand),
    /// Run any git command on every repository of the current pool.
    Command(GitCommand),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rit")]
/// Manage multiple git repositories with ease.
pub struct Application {
    #[structopt(subcommand)]
    pub command: ApplicationCommands,
}
