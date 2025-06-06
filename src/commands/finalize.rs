use crate::args::FinalizeCommand;
use crate::errors::BucketError;

pub fn execute(_p0: FinalizeCommand) -> Result<(), BucketError> {
    println!("finalize command");
    Ok(())
}