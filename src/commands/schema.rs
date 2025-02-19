use crate::args::SchemaCommand;
use crate::errors::BucketError;

pub fn execute(_command: &SchemaCommand) -> Result<(), BucketError> {
    let schema = include_str!("../sql/schema.sql");
    println!("{}", schema);
    Ok(())
} 