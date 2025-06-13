use std::io::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

use log::error;
use crate::args::RollbackCommand;
use crate::commands::commit::Commit;
use crate::utils::compression::restore_file;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::data::commit::CommitStatus;
use crate::errors::BucketError;
use crate::utils::checks;
use crate::utils::utils::{find_bucket_path, hash_file};
use crate::commands::BucketCommand;

/// Rollback command to revert changes in a bucket
pub struct Rollback {
    args: RollbackCommand,
}

impl BucketCommand for Rollback {
    type Args = RollbackCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        let current_dir = CURRENT_DIR.with(|dir| dir.clone());

        if !checks::is_valid_bucket_repo(&current_dir) {
            return Err(BucketError::NotInRepo);
        }

        let _ = match find_bucket_path(&current_dir) {
            Some(path) => path,
            None => return Err(BucketError::NotAValidBucket),
        };

        match &self.args.path {
            None => rollback_all(&current_dir),
            Some(path) => rollback_single_file(&current_dir, &path)
        }
    }
}

fn rollback_single_file(bucket_path: &PathBuf, file: &PathBuf) -> Result<(), BucketError> {
    if !file.exists() {
        return Err(BucketError::from(Error::new(ErrorKind::NotFound, "File not found.")));
    }

    let bucket = Bucket::from_meta_data(bucket_path)?;

    match Commit::load_last_commit(bucket.name) {
        Ok(None) => Err(BucketError::from(Error::new(
            ErrorKind::NotFound,
            "No previous commit found.",
        ))),
        Ok(Some(previous_commit)) => {
            let file_name = file
                .to_str()
                .ok_or_else(|| BucketError::from("Invalid UTF-8 path."))?; // Handle UTF-8 conversion error

            let file_hash = hash_file(file)?; // Properly propagate error

            let found_file = previous_commit
                .files
                .iter()
                .find(|committed_file| committed_file.name == file_name && committed_file.hash == file_hash);

            match found_file {
                None => Err(BucketError::from(Error::new(
                    ErrorKind::NotFound,
                    "File not found in previous commit.",
                ))),
                Some(file_to_restore) => {
                    restore_file(bucket_path, file_to_restore)?; // Propagate any error from restore_file
                    Ok(())
                }
            }
        }
        Err(_) => {
            error!("Failed to load previous commit.");
            Err(BucketError::from(Error::new(
                ErrorKind::Other,
                "Failed to load previous commit.",
            )))
        }
    }
}


fn rollback_all(bucket_path: &PathBuf) -> Result<(), BucketError> {
    // Read the bucket's metadata
    let bucket = Bucket::from_meta_data(&bucket_path)?;
    let bucket_files = bucket.list_files_with_metadata_in_bucket()?;
    if bucket_files.files.is_empty() {
        println!("No files in bucket");
        return Ok(());
    }

    match Commit::load_last_commit(bucket.name) {
        Ok(None) => {
            return Err(BucketError::from(Error::new(ErrorKind::NotFound, "No previous commit found.")));
        }
        Ok(Some(previous_commit)) => {
            let changes = bucket_files.compare(&previous_commit).ok_or_else(|| BucketError::from(Error::new(ErrorKind::Other, "Failed to compare files.")))?;

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
                    restore_file(&bucket_path, change).expect("Failed to restore file.");
                });
        }
        Err(_) => {
            error!("Failed to load previous commit.");
            return Err(BucketError::from(Error::new(ErrorKind::Other, "Failed to load previous commit.")));
        }
    }

    Ok(())
}
