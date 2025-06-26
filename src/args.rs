use crate::utils::checks::validate_path;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Command {
    // Repository commands
    Init(InitCommand),
    Create(CreateCommand),
    Commit(CommitCommand),
    Revert(RestoreCommand),
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

#[derive(Args, Default, Debug, Clone)]
pub struct SharedArguments {
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Clone)]
pub struct InitCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub repo_name: String,

    #[clap(long, default_value = "duckdb", value_parser = validate_database_type)]
    pub database: String,
}

fn validate_database_type(s: &str) -> Result<String, String> {
    match s.to_lowercase().as_str() {
        "duckdb" | "postgresql" | "postgres" => Ok(s.to_string()),
        _ => Err(format!("Invalid database type '{}'. Valid options are: duckdb, postgresql", s)),
    }
}

#[derive(Args, Clone)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub bucket_name: String,
}

#[derive(Args, Clone)]
pub struct CommitCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub message: String,
}

#[derive(Args, Clone)]
pub struct RestoreCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,

    #[clap(required = true)]
    pub file: String,
}

#[derive(Args, Clone)]
pub struct RollbackCommand {
    #[clap(short, long, value_name = "PATH", value_parser = validate_path)]
    pub path: Option<PathBuf>,

    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct StashCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct StatusCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Debug, Clone)]
pub struct HistoryCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct ListCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct StatsCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct ExpectCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct CheckCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct LinkCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct FinalizeCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Args, Clone)]
pub struct SchemaCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}
