use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Write};
use std::path::PathBuf;
use log::error;
use predicates::ord::le;
use zstd::stream::copy_decode;
use zstd::stream::write::Decoder;
use crate::args::RollbackCommand;
use crate::commands::commit::load_last_commit;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::data::commit::{CommitStatus, CommittedFile};
use crate::errors::BucketError;
use crate::errors::BucketError::NotInBucketsRepo;
use crate::utils::checks;
use crate::utils::utils::find_bucket_path;

pub fn execute(_rollback_command: &RollbackCommand) -> Result<(), BucketError> {
    let current_dir = CURRENT_DIR.with(|dir| dir.clone());

    if !checks::is_valid_bucket_repo(&current_dir) {
        return Err(NotInBucketsRepo);
    }

    let bucket_path = match find_bucket_path(&current_dir) {
        Some(path) => path,
        None => return Err(BucketError::NotAValidBucket),
    };

    // Read the bucket's metadata
    let bucket = Bucket::from_meta_data(&bucket_path)?;
    let bucket_files = bucket.list_files_with_metadata_in_bucket()?;
    if bucket_files.files.is_empty() {
        println!("No files in bucket");
        return Ok(());
    }

    match load_last_commit(bucket.name) {
        Ok(None) => {
            return Err(BucketError::from(Error::new(ErrorKind::NotFound, "No previous commit found.")));
        }
        Ok(Some(previous_commit)) => {
            let changes = bucket_files.compare(&previous_commit).unwrap();

            if changes
                .iter()
                .filter(|change| change.status == CommitStatus::Modified)
                .count() == 0 {
                println!("No changes detected. Rollback cancelled.");
                return Ok(());
            }

            changes
                .iter()
                .filter(|change| change.status == CommitStatus::Modified)
                .for_each(|change| {
                restore_file(&bucket_path, change).unwrap();
            });
        }
        Err(_) => {
            error!("Failed to load previous commit.");
            return Err(BucketError::from(Error::new(ErrorKind::Other, "Failed to load previous commit.")));
        }
    }


    Ok(())
}

fn restore_file(bucket_path: &PathBuf, p1: &CommittedFile) -> io::Result<()>{
    let input_path = bucket_path.join(".b").join("storage").join(&p1.previous_hash.to_string());
    let output_path = bucket_path.join(&p1.name);

    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);

    let mut decoder = Decoder::new(writer)?;
    copy_decode(reader, &mut decoder)?;
    decoder.flush()?;
    Ok(())

}