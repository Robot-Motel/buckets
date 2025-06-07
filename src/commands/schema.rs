use crate::args::SchemaCommand;
use crate::errors::BucketError;
use crate::commands::BucketCommand;

/// Schema command placeholder
pub struct Schema {
    args: SchemaCommand,
}

impl BucketCommand for Schema {
    type Args = SchemaCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!("schema command");
        Ok(())
    }
} 