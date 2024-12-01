use crate::args::HistoryCommand;
use crate::errors::BucketError;

pub fn execute(_p0: &HistoryCommand) -> Result<(), BucketError> {
    println!("history command");
    Ok(())
}