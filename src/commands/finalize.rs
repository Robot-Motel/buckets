use crate::args::FinalizeCommand;
use crate::errors::BucketError;
use crate::commands::BucketCommand;

/// Finalize command placeholder
pub struct Finalize {
    #[allow(dead_code)]
    args: FinalizeCommand,
}

impl BucketCommand for Finalize {
    type Args = FinalizeCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("finalize command");
        Ok(())
    }
}