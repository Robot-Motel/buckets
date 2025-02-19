use crate::utils::checks::validate_path;
use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Subcommand)]
pub enum Command {
    // Repository commands
    Init(InitCommand),
    Create(CreateCommand),
    Commit(CommitCommand),
    Revert(RevertCommand),
    Rollback(RollbackCommand),
    Stash(StashCommand),
    // Information commands
    Status(StatusCommand),
    History(HistoryCommand),
    List(ListCommand),
    Stats(StatsCommand),
    // Expectation commands
    Expect(ExpectCommand),
    Check(CheckCommand),
    Link(LinkCommand),
    Finalize(FinalizeCommand),
    Schema(SchemaCommand),
}

#[derive(Parser)]
#[clap(
    name = "buckets",
    version = env!("CARGO_PKG_VERSION"),
    author("3vilM33pl3"),
)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args)]
pub struct SharedArguments {
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Parser)]
pub struct InitCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub repo_name: String,
}

#[derive(Args)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub bucket_name: String,
}

#[derive(Args)]
pub struct CommitCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub message: String,
}

#[derive(Args)]
pub struct RevertCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct RollbackCommand {
    #[clap(short, long, value_name = "PATH", value_parser = validate_path)]
    pub path: Option<PathBuf>,

    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct StashCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct StatusCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct HistoryCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct ListCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct StatsCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct ExpectCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct CheckCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct LinkCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct FinalizeCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args)]
pub struct SchemaCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

