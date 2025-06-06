use crate::args::ListCommand;
use crate::errors::BucketError;

pub fn execute(_p0: ListCommand) -> Result<(), BucketError> {
    println!("list command");
    Ok(())
}