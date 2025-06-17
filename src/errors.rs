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
    NotInRepo,
    #[error("Not in a bucket")]
    NotInBucket,
    #[error("Not a valid bucket")]
    NotAValidBucket,
    #[error("Invalid bucket name: {0}")]
    InvalidBucketName(String),
    #[error("Invalid data {0}")]
    InvalidData(String),
    #[error("Not found {0}")]
    NotFound(String),
    #[error("File not found {0}")]
    FileNotFound(String),
}

impl BucketError {
    pub(crate) fn message(&self) -> String {
        match self {
            BucketError::IoError(e) => format!("IO Error: {}", e),
            BucketError::DuckDB(e) => format!("Database Error: {}", e),
            BucketError::BucketAlreadyExists => "Bucket already exists".to_string(),
            BucketError::RepoAlreadyExists(message) => {
                format!("Repository {} already exists", message)
            }
            BucketError::NotInRepo => "Not in a buckets repository".to_string(),
            BucketError::NotInBucket => "Not in a bucket".to_string(),
            // BucketError::InBucketRepo => "Already in a bucket repository".to_string(),
            BucketError::NotAValidBucket => "Not a valid bucket".to_string(),
            BucketError::InvalidBucketName(message) => format!("Invalid bucket name: {}", message),
            BucketError::InvalidData(message) => format!("Invalid data {}", message),
            BucketError::NotFound(message) => format!("Not found {}", message),
            BucketError::FileNotFound(message) => format!("File not found {}", message),
        }
    }
}

impl From<&str> for BucketError {
    fn from(error: &str) -> Self {
        BucketError::IoError(io::Error::new(io::ErrorKind::Other, error))
    }
}

impl From<BucketError> for duckdb::Error {
    fn from(error: BucketError) -> duckdb::Error {
        duckdb::Error::ToSqlConversionFailure(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_bucket_error_message_formatting() {
        assert_eq!(
            BucketError::BucketAlreadyExists.message(),
            "Bucket already exists"
        );
        assert_eq!(
            BucketError::RepoAlreadyExists("test_repo".to_string()).message(),
            "Repository test_repo already exists"
        );
        assert_eq!(
            BucketError::NotInRepo.message(),
            "Not in a buckets repository"
        );
        assert_eq!(BucketError::NotInBucket.message(), "Not in a bucket");
        assert_eq!(BucketError::NotAValidBucket.message(), "Not a valid bucket");
        assert_eq!(
            BucketError::InvalidBucketName("cannot be empty".to_string()).message(),
            "Invalid bucket name: cannot be empty"
        );
        assert_eq!(
            BucketError::InvalidData("corrupted".to_string()).message(),
            "Invalid data corrupted"
        );
        assert_eq!(
            BucketError::NotFound("file.txt".to_string()).message(),
            "Not found file.txt"
        );
        assert_eq!(
            BucketError::FileNotFound("missing.txt".to_string()).message(),
            "File not found missing.txt"
        );
    }

    #[test]
    fn test_bucket_error_display() {
        let error = BucketError::NotInRepo;
        assert_eq!(format!("{}", error), "Not in a buckets repository");

        let error = BucketError::InvalidBucketName("test".to_string());
        assert_eq!(format!("{}", error), "Invalid bucket name: test");
    }

    #[test]
    fn test_error_from_str_conversion() {
        let error: BucketError = "test error message".into();
        match error {
            BucketError::IoError(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::Other);
                assert_eq!(io_err.to_string(), "test error message");
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let bucket_error: BucketError = io_error.into();
        match bucket_error {
            BucketError::IoError(err) => {
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(err.to_string(), "file not found");
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_to_duckdb_conversion() {
        let bucket_error = BucketError::NotInRepo;
        let duckdb_error: duckdb::Error = bucket_error.into();
        match duckdb_error {
            duckdb::Error::ToSqlConversionFailure(_) => {
                // Conversion successful
            }
            _ => panic!("Expected ToSqlConversionFailure variant"),
        }
    }

    #[test]
    fn test_io_error_message() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let bucket_error = BucketError::IoError(io_error);
        assert!(bucket_error.message().contains("IO Error:"));
        assert!(bucket_error.message().contains("access denied"));
    }

    #[test]
    fn test_error_debug_format() {
        let error = BucketError::BucketAlreadyExists;
        let debug_str = format!("{:?}", error);
        assert_eq!(debug_str, "BucketAlreadyExists");

        let error = BucketError::InvalidBucketName("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidBucketName"));
        assert!(debug_str.contains("test"));
    }
}
