use crate::args::StatsCommand;
use crate::errors::BucketError;

pub fn execute(_p0: StatsCommand) -> Result<(), BucketError> {
    println!("stats command");
    Ok(())
}