use chrono::Utc;
use duckdb::Connection;
use log::error;
use uuid::Uuid;
use BucketError::NotInBucketsRepo;
use crate::args::CreateCommand;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::errors::BucketError;
use crate::utils::checks;
use crate::utils::checks::{find_directory_in_parents, is_valid_bucket};

pub fn execute(create_command: &CreateCommand) -> Result<(), BucketError> {
    let bucket_name = &create_command.bucket_name;

    checks(&bucket_name)?;

    let bucket_path = CURRENT_DIR.with(|dir| dir.join(&bucket_name));
    std::fs::create_dir_all(&bucket_path.join(".b").join("storage"))?;

    let buckets_repo_path = find_directory_in_parents(&bucket_path, ".buckets").unwrap();
    let relative_path = match bucket_path.strip_prefix(&buckets_repo_path.parent().unwrap()) {
        Ok(x) => x,
        Err(_) => {
            return Err(BucketError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error stripping prefix",
            )))
        }
    }.to_path_buf();

    let db_path = buckets_repo_path.join("buckets.db");
    let connection = Connection::open(db_path)?;
    let timestamp = Utc::now().to_rfc3339();

    match connection
        .execute(
            "INSERT INTO buckets (id, name, path, created_at) VALUES (gen_random_uuid(), ?1, ?2, ?3)",
            &[&bucket_name, relative_path.to_str().unwrap(), &timestamp],
        )
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error inserting into database: {}", e),
            )
        }) {
        Ok(_) => {}
        Err(e) => {
            error!("Error inserting into database: {}", e);
            return Err(e.into());
        }
    }

    let mut stmt = connection
        .prepare("SELECT id FROM buckets WHERE name = ?1 AND path = ?2")
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error preparing statement: {}", e),
            )
        })?;

    let bucket_id_str: String = stmt
        .query_row(&[&bucket_name, relative_path.to_str().unwrap()], |row| {
            row.get(0)
        })
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error querying statement: {}", e),
            )
        })?;

    let bucket_id = Uuid::parse_str(&bucket_id_str).unwrap();
    let bucket = Bucket::default(bucket_id, bucket_name, &relative_path);
    bucket.write_bucket_info();

    Ok(())
}

fn checks(bucket_name: &str) -> Result<(), BucketError> {
    let bucket_location = CURRENT_DIR.with(|dir| dir.join(bucket_name));
    let current_dir = CURRENT_DIR.with(|dir| dir.clone());

    // Check if in valid buckets repository
    if !checks::is_valid_bucket_repo(&current_dir) {
        return Err(NotInBucketsRepo);
    }

    if bucket_location.exists() {
        if bucket_location.is_dir() && is_valid_bucket(&bucket_location) {
            return Err(BucketError::BucketAlreadyExists);
        } else if bucket_location.is_file() {
            return  Err(BucketError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File with the same name already exists",
            )));
        } else if bucket_location.is_dir() {
            return Err(BucketError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Directory already exists",
            )));
        } else { return Err(BucketError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown error",
        )));
        }
    }

    Ok(())

}