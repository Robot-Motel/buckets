use std::env;
use std::io::{Error, ErrorKind};
use duckdb::Connection;
use log::error;
use crate::args::StatusCommand;
use crate::commands::commit::load_last_commit;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::errors::BucketError;
use crate::errors::BucketError::NotInBucketsRepo;
use crate::utils::checks;
use crate::utils::config::RepositoryConfig;
use crate::utils::utils::{find_bucket_path, find_directory_in_parents};

pub fn execute(_status_command: &StatusCommand) -> Result<(), BucketError> {

    let current_dir = CURRENT_DIR.with(|dir| dir.clone());

    if !checks::is_valid_bucket_repo(&current_dir) {
        return Err(NotInBucketsRepo);
    }

    if !checks::is_valid_bucket(&current_dir) {
        repository_status()
    } else {
        let bucket_path = find_bucket_path(&current_dir).expect("this _should_ be a valid bucket");

        let bucket = match Bucket::from_meta_data(&bucket_path) {
            Ok(bucket) => bucket,
            Err(e) => {
                error!("Error reading bucket info: {}", e);
                return Err(e);
            }
        };
        bucket_status(&bucket)
    }.expect("TODO: panic message");

    Ok(())
}

fn bucket_status(bucket: &Bucket) -> Result<(), BucketError> {
    // Read the bucket's metadata
    let bucket = Bucket::from_meta_data(&bucket.get_full_bucket_path()?)?;
    let bucket_files = bucket.list_files_with_metadata_in_bucket()?;
    if bucket_files.files.is_empty() {
        println!("No files in bucket");
        return Ok(());
    }

    match load_last_commit(bucket.name) {
        Ok(None) => {
            bucket_files.files.iter().for_each(|file| {
                println!("new file:    {}", file.name );
            });
        }
        Ok(Some(previous_commit)) => {
            let changes = bucket_files.compare(&previous_commit).ok_or_else(|| BucketError::from("Failed to compare files."))?;
            changes.iter().for_each(|change| {
                println!("{}:    {}", change.status, change.name);
            });
        }
        Err(_) => {
            error!("Failed to load previous commit.");
            return Err(BucketError::from(Error::new(ErrorKind::Other, "Failed to load previous commit.")));
        }
    }



    Ok(())
}

fn repository_status() -> Result<(), BucketError> {
    let repo_config = RepositoryConfig::from_file(env::current_dir().expect("invalid dir"))?;
    println!("Repository config: {:?}", repo_config);
    let buckets = query_buckets().map_err(|e| BucketError::from(e))?;
    println!("Number of buckets: {:?}", buckets.len());
    println!("Buckets: {:?}", buckets);
    Ok(())
}

fn query_buckets() -> Result<Vec<Bucket>, BucketError> {
    let db_path = find_directory_in_parents(&env::current_dir().expect("invalid dir"), ".buckets").expect("invalid dir").join("buckets.db");
    let connection = Connection::open(db_path).expect("failed to open database");
    let mut stmt = connection.prepare("SELECT id, name, path FROM buckets").map_err(|e| BucketError::from(e))?;
    let bucket_iter = stmt.query_map([], |row| {
        let uuid_str: String = row.get(0)?;
        let path_str: String = row.get(2)?;
        let uuid = uuid::Uuid::parse_str(&uuid_str)
            .map_err(|e| BucketError::InvalidData(e.to_string()))?;
        Ok(Bucket {
            id: uuid,
            name: row.get(1)?,
            relative_bucket_path: std::path::PathBuf::from(path_str),
        })
    }).map_err(BucketError::from)?;
    let mut buckets = Vec::new();
    for bucket in bucket_iter {
        buckets.push(bucket.map_err(BucketError::from)?); // Ensure all errors are converted properly
    }

    Ok(buckets)
}