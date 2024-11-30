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

static ARGS: Lazy<CliArguments> = Lazy::new(|| {
    CliArguments::try_parse().unwrap_or_else(|error| {
        if error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
            println!("Please provide a subcommand: {}", error);
        }
        error.exit();
    })
});

// Define the thread-local EXIT variable with initial value of SUCCESS
thread_local! {
    static EXIT: Cell<ExitCode> = Cell::new(ExitCode::SUCCESS);
    static CURRENT_DIR: PathBuf = std::env::current_dir().unwrap();
}

// Function to set the exit code to failure
fn set_failed() {
    EXIT.with(|cell| cell.set(ExitCode::FAILURE));
}

fn main() -> ExitCode {
    let res = dispatch();

    if let Err(msg) = res {
        set_failed();
        println!("{}", msg.message());
    }

    EXIT.with(|cell| cell.get())
}

fn dispatch() -> Result<(), BucketError> {
    match &ARGS.command {
        Command::Init(command) => commands::init::execute(command)?,
        Command::Status(command) => commands::status::execute(command)?,
        Command::Create(command) => commands::create::execute(command)?,
        Command::Commit(command) => commands::commit::execute(command)?,
    }

    Ok(())
}