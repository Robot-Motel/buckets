use crate::args::ExpectCommand;
use crate::errors::BucketError;

pub fn execute(_p0: ExpectCommand) -> Result<(), BucketError> {
    println!("expect command");
    Ok(())
}