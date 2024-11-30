use crate::args::CommitCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &CommitCommand) -> Result<(), BucketError> {
    println!("commit command");
    Ok(())
}