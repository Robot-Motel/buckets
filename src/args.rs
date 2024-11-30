use clap::{Args, Parser, Subcommand};

#[derive(Subcommand)]
pub enum Command {
    Init(InitCommand),
    Create(CreateCommand),
    Commit(CommitCommand),
    Status(StatusCommand),
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

    #[clap(short, long, required = false)]
    pub message: Option<String>,

}

#[derive(Args)]
pub struct StatusCommand {
    #[clap(flatten)]
    pub shared: SharedArguments,
}

#[derive(Parser)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args)]
pub struct SharedArguments {
    #[clap(short, long)]
    pub verbose: bool,
}