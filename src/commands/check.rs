use crate::args::CheckCommand;
use crate::commands::BucketCommand;
use crate::errors::BucketError;

/// Check command placeholder
pub struct Check {
    #[allow(dead_code)]
    args: CheckCommand,
}

impl BucketCommand for Check {
    type Args = CheckCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("check command");
        Ok(())
    }
}
