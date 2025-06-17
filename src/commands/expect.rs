use crate::args::ExpectCommand;
use crate::commands::BucketCommand;
use crate::errors::BucketError;

/// Expect command placeholder
pub struct Expect {
    #[allow(dead_code)]
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
