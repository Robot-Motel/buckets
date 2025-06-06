use crate::args::LinkCommand;
use crate::errors::BucketError;

pub fn execute(_p0: LinkCommand) -> Result<(), BucketError> {
    println!("link command");
    Ok(())
}