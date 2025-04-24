use std::cell::Cell;
use std::path::PathBuf;
use std::process::ExitCode;
use clap::error::ErrorKind;
use clap::Parser;
use once_cell::sync::Lazy;
use crate::args::{CliArguments, Command};
use crate::errors::BucketError;

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
        Command::Init(command) => commands::init::execute(command)?,
        Command::Create(command) => commands::create::execute(command)?,
        Command::Commit(command) => commands::commit::execute(command)?,
        Command::Revert(command) => commands::restore::execute(command)?,
        Command::Rollback(command) => commands::rollback::execute(command)?,
        Command::Stash(command) => commands::stash::execute(command)?,
        // Informational commands
        Command::Status(command) => commands::status::execute(command)?,
        Command::History(command) => commands::history::execute(command)?,
        Command::List(command) => commands::list::execute(command)?,
        Command::Stats(command) => commands::stats::execute(command)?,
        // Expectation commands
        Command::Expect(command) => commands::expect::execute(command)?,
        Command::Check(command) => commands::check::execute(command)?,
        Command::Link(command) => commands::link::execute(command)?,
        Command::Finalize(command) => commands::finalize::execute(command)?,
        Command::Schema(command) => commands::schema::execute(command)?,
    }

    Ok(())
}