use crate::args::ExpectCommand;
use crate::errors::BucketError;
use crate::commands::BucketCommand;

/// Expect command placeholder
pub struct Expect {
    args: ExpectCommand,
}

impl BucketCommand for Expect {
    type Args = ExpectCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("expect command");
        Ok(())
    }
}