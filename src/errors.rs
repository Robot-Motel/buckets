use std::io;

pub enum BucketError {
    #[allow(dead_code)]
    IoError(io::Error),
    // DuckDB(duckdb::Error),
    // #[allow(dead_code)]
    // BucketAlreadyExists,
    // RepoAlreadyExists(String),
    // #[allow(dead_code)]
    // NotInBucketRepo,
    // #[allow(dead_code)]
    // InBucketRepo,
    // NotAValidBucket,
}

impl BucketError {
    pub(crate) fn message(&self) -> String {
        todo!()
    }
}