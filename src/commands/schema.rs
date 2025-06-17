use crate::args::SchemaCommand;
use crate::commands::BucketCommand;
use crate::errors::BucketError;

/// Schema command placeholder
pub struct Schema {
    #[allow(dead_code)]
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
