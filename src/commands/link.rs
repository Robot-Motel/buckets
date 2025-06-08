use crate::args::LinkCommand;
use crate::errors::BucketError;
use crate::commands::BucketCommand;

/// Link command placeholder
pub struct Link {
    #[allow(dead_code)]
    args: LinkCommand,
}

impl BucketCommand for Link {
    type Args = LinkCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("link command");
        Ok(())
    }
}