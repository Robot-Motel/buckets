use crate::args::CreateCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &CreateCommand) -> Result<(), BucketError> {
    println!("create command");
    Ok(())
}