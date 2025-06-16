use chrono::Utc;
use duckdb::Connection;
use log::error;
use uuid::Uuid;
use crate::args::CreateCommand;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::errors::BucketError;
use crate::utils::checks;
use crate::utils::checks::{find_directory_in_parents, is_valid_bucket};
use crate::commands::BucketCommand;

/// Create a new bucket
pub struct Create {
    args: CreateCommand,
}

impl BucketCommand for Create {
    type Args = CreateCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        let bucket_name = &self.args.bucket_name;

        self.checks(&bucket_name)?;

    let bucket_path = CURRENT_DIR.with(|dir| dir.join(&bucket_name));
    std::fs::create_dir_all(&bucket_path.join(".b").join("storage"))?;

    let buckets_repo_path = find_directory_in_parents(&bucket_path, ".buckets").ok_or_else(|| BucketError::NotInRepo)?;
    let relative_path = match bucket_path.strip_prefix(&buckets_repo_path.parent().ok_or_else(|| BucketError::NotInRepo)?) {
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
            [bucket_name, relative_path.to_str().ok_or_else(|| 
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid path string"))?, &timestamp],
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
        .query_row([bucket_name, relative_path.to_str().ok_or_else(|| 
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid path string"))?], |row| {
            row.get(0)
        })
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error querying statement: {}", e),
            )
        })?;

    let bucket_id = Uuid::parse_str(&bucket_id_str).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing UUID: {}", e),
        )
    })?;
    let bucket = Bucket::default(bucket_id, bucket_name, &relative_path);
    bucket.write_bucket_info().map_err(|e| BucketError::from(e))?;

    Ok(())
    }
}

impl Create {
    fn checks(&self, bucket_name: &str) -> Result<(), BucketError> {
    let bucket_location = CURRENT_DIR.with(|dir| dir.join(bucket_name));
    let current_dir = CURRENT_DIR.with(|dir| dir.clone());

    // Check if in valid buckets repository
    if !checks::is_valid_bucket_repo(&current_dir) {
        return Err(BucketError::NotInRepo);
    }

    // Validate bucket name
    if bucket_name.is_empty() {
        return Err(BucketError::InvalidBucketName("cannot be empty".to_string()));
    }
    
    if bucket_name == "." || bucket_name == ".." {
        return Err(BucketError::InvalidBucketName("cannot be '.' or '..'".to_string()));
    }
    
    if bucket_name.contains('/') || bucket_name.contains('\\') {
        return Err(BucketError::InvalidBucketName("cannot contain path separators".to_string()));
    }
    
    if bucket_name.contains('\0') {
        return Err(BucketError::InvalidBucketName("cannot contain null characters".to_string()));
    }
    
    if bucket_name.len() > 255 {
        return Err(BucketError::InvalidBucketName("too long (maximum 255 characters)".to_string()));
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
}
