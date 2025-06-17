use crate::args::StashCommand;
use crate::commands::BucketCommand;
use crate::errors::BucketError;

/// Stash command placeholder
pub struct Stash {
    #[allow(dead_code)]
    args: StashCommand,
}

impl BucketCommand for Stash {
    type Args = StashCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("stash command");
        Ok(())
    }
}
