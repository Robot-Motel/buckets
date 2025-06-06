use std::cell::Cell;
use std::path::PathBuf;
use std::process::ExitCode;
use clap::error::ErrorKind;
use clap::Parser;
use once_cell::sync::Lazy;
use crate::args::{CliArguments, Command};
use crate::errors::BucketError;
use crate::commands::BucketCommand;

mod args;
mod commands;
mod errors;
mod utils;
mod config;
mod data;
mod world;

static ARGS: Lazy<CliArguments> = Lazy::new(|| {
    CliArguments::try_parse().unwrap_or_else(|error| {
        if error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
            println!("Please provide a subcommand: {}", error);
            std::process::exit(0); // Exit with success code after showing help
        }
        error.exit();
    })
});

// Define the thread-local EXIT variable with initial value of SUCCESS
thread_local! {
    static EXIT: Cell<ExitCode> = Cell::new(ExitCode::SUCCESS);
    static CURRENT_DIR: PathBuf = std::env::current_dir().expect("Failed to get current directory.");
}

// Function to set the exit code to failure
fn set_failed() {
    EXIT.with(|cell| cell.set(ExitCode::FAILURE));
}

fn main() -> ExitCode {
    let res = dispatch();

    if let Err(msg) = res {
        set_failed();
        eprintln!("{}", msg.message());
    }

    EXIT.with(|cell| cell.get())
}

fn dispatch() -> Result<(), BucketError> {

    match &ARGS.command {
        // Commands that modify the repository
        Command::Init(command) => commands::init::Init::new(command).execute()?,
        Command::Create(command) => commands::create::Create::new(command).execute()?,
        Command::Commit(command) => commands::commit::execute(command.clone())?,
        Command::Revert(command) => commands::restore::execute(command.clone())?,
        Command::Rollback(command) => commands::rollback::execute(command.clone())?,
        Command::Stash(command) => commands::stash::execute(command.clone())?,
        // Informational commands
        Command::Status(command) => commands::status::execute(command.clone())?,
        Command::History(command) => commands::history::execute(command.clone())?,
        Command::List(command) => commands::list::execute(command.clone())?,
        Command::Stats(command) => commands::stats::execute(command.clone())?,
        // Expectation commands
        Command::Expect(command) => commands::expect::execute(command.clone())?,
        Command::Check(command) => commands::check::execute(command.clone())?,
        Command::Link(command) => commands::link::execute(command.clone())?,
        Command::Finalize(command) => commands::finalize::execute(command.clone())?,
        Command::Schema(command) => commands::schema::execute(command.clone())?,
    }

    Ok(())
}