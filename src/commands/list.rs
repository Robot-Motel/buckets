use crate::args::ListCommand;
use crate::commands::BucketCommand;
use crate::errors::BucketError;

/// List command placeholder
pub struct List {
    #[allow(dead_code)]
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
