use std::fmt::{Display, Formatter};
use std::io;

pub enum BucketError {
    IoError(io::Error),
    DuckDB(duckdb::Error),
    BucketAlreadyExists,
    RepoAlreadyExists(String),
    NotInBucketsRepo,
    // #[allow(dead_code)]
    // InBucketRepo,
    NotAValidBucket,
}

impl BucketError {
    pub(crate) fn message(&self) -> String {
        match self {
            BucketError::IoError(e) => format!("IO Error: {}", e),
            BucketError::DuckDB(e) => format!("Database Error: {}", e),
            BucketError::BucketAlreadyExists => "Bucket already exists".to_string(),
            BucketError::RepoAlreadyExists(message) => format!("Repository {} already exists", message),
            BucketError::NotInBucketsRepo => "Not in a bucket repository".to_string(),
            // BucketError::InBucketRepo => "Already in a bucket repository".to_string(),
            BucketError::NotAValidBucket => "Not a valid bucket".to_string(),
        }
    }
}

impl Display for BucketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BucketError::IoError(e) => write!(f, "IO Error: {}", e),
            BucketError::DuckDB(e) => write!(f, "Database Error: {}", e),
            BucketError::BucketAlreadyExists => write!(f, "Bucket already exists"),
            BucketError::RepoAlreadyExists(message) => write!(f, "Repository already exists {}", message),
            BucketError::NotInBucketsRepo => write!(f, "Not in a bucket repository"),
            // BucketError::InBucketRepo => write!(f, "Already in a bucket repository"),
            BucketError::NotAValidBucket => write!(f, "Not a valid bucket"),
        }
    }
}

impl From<io::Error> for BucketError {
    fn from(error: io::Error) -> Self {
        BucketError::IoError(error)
    }
}

impl From<duckdb::Error> for BucketError {
    fn from(error: duckdb::Error) -> Self {
        // BucketError::DuckDB(error)
        BucketError::IoError(io::Error::new(io::ErrorKind::Other, error.to_string()))
    }

}