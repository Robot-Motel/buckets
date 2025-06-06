use crate::args::ListCommand;
use crate::errors::BucketError;
use crate::commands::BucketCommand;

/// List command placeholder
pub struct List {
    args: ListCommand,
}

impl BucketCommand for List {
    type Args = ListCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("list command");
        Ok(())
    }
}

// Keep the old function for backward compatibility during transition
pub fn execute(_p0: ListCommand) -> Result<(), BucketError> {
    let cmd = List::new(&_p0);
    cmd.execute()
}