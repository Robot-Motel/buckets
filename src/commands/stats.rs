use crate::args::StatsCommand;
use crate::commands::BucketCommand;
use crate::errors::BucketError;

/// Stats command placeholder
pub struct Stats {
    #[allow(dead_code)]
    args: StatsCommand,
}

impl BucketCommand for Stats {
    type Args = StatsCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("stats command");
        Ok(())
    }
}
