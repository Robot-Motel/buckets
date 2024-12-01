use crate::args::RollbackCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &RollbackCommand) -> Result<(), BucketError> {
    println!("rollback command");
    Ok(())
}