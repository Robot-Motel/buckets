use crate::args::{CliArguments, Command};
use crate::commands::BucketCommand;
use crate::errors::BucketError;
use clap::error::ErrorKind;
use clap::Parser;
use once_cell::sync::Lazy;
use std::cell::Cell;
use std::path::PathBuf;
use std::process::ExitCode;

mod args;
mod commands;
mod config;
mod data;
mod database;
mod errors;
mod utils;
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
    static CURRENT_DIR: PathBuf = std::env::current_dir().unwrap_or_else(|_| {
        eprintln!("Error: Failed to get current directory. Using current directory as fallback.");
        PathBuf::from(".")
    });
}

// Function to set the exit code to failure
fn set_failed() {
    EXIT.with(|cell| cell.set(ExitCode::FAILURE));
}

fn main() -> ExitCode {
    let res = dispatch();

    if let Err(msg) = res {
        set_failed();
        eprintln!("{}", msg);
    }

    EXIT.with(|cell| cell.get())
}

fn dispatch() -> Result<(), BucketError> {
    match &ARGS.command {
        // Commands that modify the repository
        Command::Init(command) => commands::init::Init::new(command).execute()?,
        Command::Create(command) => commands::create::Create::new(command).execute()?,
        Command::Commit(command) => commands::commit::Commit::new(command).execute()?,
        Command::Revert(command) => commands::restore::Restore::new(command).execute()?,
        Command::Rollback(command) => commands::rollback::Rollback::new(command).execute()?,
        Command::Stash(command) => commands::stash::Stash::new(command).execute()?,
        // Informational commands
        Command::Status(command) => commands::status::Status::new(command).execute()?,
        Command::History(command) => commands::history::execute(command.clone())?,
        Command::List(command) => commands::list::List::new(command).execute()?,
        Command::Stats(command) => commands::stats::Stats::new(command).execute()?,
        // Expectation commands
        Command::Expect(command) => commands::expect::Expect::new(command).execute()?,
        Command::Check(command) => commands::check::Check::new(command).execute()?,
        Command::Link(command) => commands::link::Link::new(command).execute()?,
        Command::Finalize(command) => commands::finalize::Finalize::new(command).execute()?,
        Command::Schema(command) => commands::schema::Schema::new(command).execute()?,
    }

    Ok(())
}
