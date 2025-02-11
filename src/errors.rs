use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BucketError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
    #[error("Database Error: {0}")]
    DuckDB(#[from] duckdb::Error),
    #[error("Bucket already exists")]
    BucketAlreadyExists,
    #[error("Repository {0} already exists")]
    RepoAlreadyExists(String),
    #[error("Not in a buckets repository")]
    NotInBucketsRepo,
    #[error("Not a valid bucket")]
    NotAValidBucket,
}

impl BucketError {
    pub(crate) fn message(&self) -> String {
        match self {
            BucketError::IoError(e) => format!("IO Error: {}", e),
            BucketError::DuckDB(e) => format!("Database Error: {}", e),
            BucketError::BucketAlreadyExists => "Bucket already exists".to_string(),
            BucketError::RepoAlreadyExists(message) => format!("Repository {} already exists", message),
            BucketError::NotInBucketsRepo => "Not in a buckets repository".to_string(),
            // BucketError::InBucketRepo => "Already in a bucket repository".to_string(),
            BucketError::NotAValidBucket => "Not a valid bucket".to_string(),
        }
    }
}

impl From<&str> for BucketError {
    fn from(error: &str) -> Self {
        BucketError::IoError(io::Error::new(io::ErrorKind::Other, error))
    }
}