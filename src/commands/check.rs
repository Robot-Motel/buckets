use crate::args::CheckCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &CheckCommand) -> Result<(), BucketError> {
    println!("check command");
    Ok(())
}