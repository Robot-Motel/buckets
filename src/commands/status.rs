use crate::args::StatusCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &StatusCommand) -> Result<(), BucketError> {
    println!("status command");
    Ok(())
}