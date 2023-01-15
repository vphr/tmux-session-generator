
use clap::{Parser, Args, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct TmuxArgs {
    #[clap(subcommand)]
    pub action: Option<Action>
}

#[derive(Debug, Subcommand)]
pub enum Action{
    /// Provide an active session name to use for the current directory.
    Create(CreateArgs),
    /// (TEMPORARY): Provide a path to generated configuration.
    Load(LoadArgs),
    Update,
}


#[derive(Debug, Args)]
pub struct CreateArgs{
    pub session_name: String
}

#[derive(Debug, Args)]
pub struct LoadArgs{
    pub path: String
}
