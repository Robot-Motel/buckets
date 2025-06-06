use crate::args::StashCommand;
use crate::errors::BucketError;

pub fn execute(_p0: StashCommand) -> Result<(), BucketError> {
    println!("stash command");
    Ok(())
}