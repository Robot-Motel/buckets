use crate::args::RevertCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &RevertCommand) -> Result<(), BucketError> {
    println!("revert command");
    Ok(())
}