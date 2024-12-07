use std::io;

pub enum BucketError {
    #[allow(dead_code)]
    IoError(io::Error),
    // DuckDB(duckdb::Error),
    // #[allow(dead_code)]
    // BucketAlreadyExists,
    RepoAlreadyExists(String),
    // #[allow(dead_code)]
    // NotInBucketRepo,
    // #[allow(dead_code)]
    // InBucketRepo,
    // NotAValidBucket,
}

impl BucketError {
    pub(crate) fn message(&self) -> String {
        match self {
            BucketError::IoError(e) => format!("IO Error: {}", e),
            // BucketError::DuckDB(e) => format!("Database Error: {}", e),
            // BucketError::BucketAlreadyExists => "Bucket already exists".to_string(),
            BucketError::RepoAlreadyExists(message) => format!("Repository {} already exists", message),
            // BucketError::NotInBucketRepo => "Not in a bucket repository".to_string(),
            // BucketError::InBucketRepo => "Already in a bucket repository".to_string(),
            // BucketError::NotAValidBucket => "Not a valid bucket".to_string(),
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