use std::path::PathBuf;

use crate::{
    args::SharedArguments,
    data::bucket::{Bucket, BucketTrait},
    errors::BucketError,
    utils::{checks, utils::find_bucket_repo},
    CURRENT_DIR,
};

pub struct World {
    // Path to the working directory
    pub work_dir: PathBuf,
    // The root directory of the repository
    pub repo_root: PathBuf,
    // Path to the database file
    pub repo_db_path: PathBuf,
    // The active bucket, None if no bucket is active
    pub bucket: Option<Bucket>,
    // Verbose output
    pub verbose: bool,
}

impl World {
    pub fn new(args: &SharedArguments) -> Result<Self, BucketError> {
        let work_dir = CURRENT_DIR.with(|dir| dir.clone());

        if !checks::is_valid_bucket_repo(&work_dir) {
            return Err(BucketError::NotInRepo);
        }

        let repo_root = match find_bucket_repo(&work_dir) {
            Some(path) => path,
            None => return Err(BucketError::NotInRepo),
        };

        let repo_db_path = repo_root.join(".buckets").join("buckets.db");

        let bucket = match Bucket::from_meta_data(&work_dir) {
            Ok(bucket) => Some(bucket),
            Err(e) => None,
        };

        let verbose = args.verbose;

        Ok(World {
            work_dir,
            repo_root,
            repo_db_path,
            bucket,
            verbose,
        })
    }
}
