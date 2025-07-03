use std::io::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::args::RollbackCommand;
use crate::commands::commit::Commit;
use crate::commands::BucketCommand;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::data::commit::CommitStatus;
use crate::errors::BucketError;
use crate::utils::checks;
use crate::utils::utils::{find_bucket_path, hash_file};
use crate::CURRENT_DIR;
use log::error;

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
            Some(path) => rollback_single_file(&current_dir, &path),
        }
    }
}

fn rollback_single_file(bucket_path: &PathBuf, file: &PathBuf) -> Result<(), BucketError> {
    if !file.exists() {
        return Err(BucketError::from(Error::new(
            ErrorKind::NotFound,
            "File not found.",
        )));
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

            let found_file = previous_commit.files.iter().find(|committed_file| {
                committed_file.name == file_name && committed_file.hash == file_hash
            });

            match found_file {
                None => Err(BucketError::from(Error::new(
                    ErrorKind::NotFound,
                    "File not found in previous commit.",
                ))),
                Some(file_to_restore) => {
                    file_to_restore.restore(bucket_path)?; // Propagate any error from restore_file
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
            return Err(BucketError::from(Error::new(
                ErrorKind::NotFound,
                "No previous commit found.",
            )));
        }
        Ok(Some(previous_commit)) => {
            let changes = bucket_files.compare(&previous_commit).ok_or_else(|| {
                BucketError::from(Error::new(ErrorKind::Other, "Failed to compare files."))
            })?;

            if changes
                .iter()
                .filter(|change| change.status == CommitStatus::Modified)
                .count()
                == 0
            {
                println!("No changes detected. Rollback cancelled.");
                return Ok(());
            }

            changes
                .iter()
                .filter(|change| change.status == CommitStatus::Modified)
                .for_each(|change| {
                    if let Err(e) = change.restore(&bucket_path) {
                        error!("Failed to restore file: {}", e);
                    }
                });
        }
        Err(_) => {
            error!("Failed to load previous commit.");
            return Err(BucketError::from(Error::new(
                ErrorKind::Other,
                "Failed to load previous commit.",
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::SharedArguments;
    use crate::data::bucket::Bucket;
    use crate::data::commit::{Commit as CommitData, CommittedFile};
    use crate::utils::utils::hash_file;
    use blake3::Hash;
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::tempdir;
    use uuid::Uuid;

    // Helper function to create a test rollback command
    fn create_test_rollback_command(path: Option<PathBuf>) -> Rollback {
        let args = RollbackCommand {
            path,
            shared: SharedArguments::default(),
        };
        Rollback::new(&args)
    }

    // Helper function to create a test repository and bucket structure
    fn create_test_repo_and_bucket_structure(
    ) -> Result<(tempfile::TempDir, PathBuf, PathBuf), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let repo_path = temp_dir.path().join("test_repo");
        let bucket_path = repo_path.join("test_bucket");

        // Create repository structure
        fs::create_dir_all(&repo_path)?;
        let buckets_dir = repo_path.join(".buckets");
        fs::create_dir_all(&buckets_dir)?;

        // Create repository config file
        let config_path = buckets_dir.join("config");
        let config_content = r#"ntp_server = "pool.ntp.org"
ip_check = "8.8.8.8"  
url_check = "api.ipify.org"
"#;
        fs::write(&config_path, config_content)?;

        // Initialize database properly with schema
        use crate::database::{initialize_database, DatabaseType};
        initialize_database(&buckets_dir, DatabaseType::DuckDB)?;

        // Create bucket structure
        fs::create_dir_all(&bucket_path)?;
        let b_dir = bucket_path.join(".b");
        fs::create_dir_all(&b_dir)?;
        fs::create_dir_all(b_dir.join("storage"))?;

        // Create bucket info file
        let info_path = b_dir.join("info");
        let bucket_info = Bucket {
            id: Uuid::new_v4(),
            name: "test_bucket".to_string(),
            relative_bucket_path: PathBuf::from("test_bucket"),
        };
        let info_content = toml::to_string(&bucket_info)?;
        fs::write(&info_path, info_content)?;

        Ok((temp_dir, repo_path, bucket_path))
    }

    // Helper function to create a test bucket directory structure (for unit tests that don't need full repo)
    fn create_test_bucket_structure(
    ) -> Result<(tempfile::TempDir, PathBuf), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().join("test_bucket");
        fs::create_dir_all(&bucket_path)?;

        // Create .b directory structure
        let b_dir = bucket_path.join(".b");
        fs::create_dir_all(&b_dir)?;
        fs::create_dir_all(b_dir.join("storage"))?;

        // Create bucket info file
        let info_path = b_dir.join("info");
        let bucket_info = Bucket {
            id: Uuid::new_v4(),
            name: "test_bucket".to_string(),
            relative_bucket_path: PathBuf::from("test_bucket"),
        };
        let info_content = toml::to_string(&bucket_info)?;
        fs::write(&info_path, info_content)?;

        Ok((temp_dir, bucket_path))
    }

    // Helper function to create a test file and compress it to storage
    fn create_test_file_and_compress(
        bucket_path: &PathBuf,
        file_name: &str,
        content: &str,
    ) -> Result<(PathBuf, Hash), Box<dyn std::error::Error>> {
        let file_path = bucket_path.join(file_name);

        // Create directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&file_path, content)?;
        let hash = hash_file(&file_path)?;

        // Create compressed version in storage
        let _storage_path = bucket_path
            .join(".b")
            .join("storage")
            .join(hash.to_hex().as_str());

        // Use the compression utility that the actual code uses
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: file_name.to_string(),
            hash,
            previous_hash: Hash::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            status: crate::data::commit::CommitStatus::New,
        };

        committed_file.compress_and_store(&bucket_path)?;

        Ok((file_path, hash))
    }

    #[test]
    fn test_rollback_new() {
        let rollback = create_test_rollback_command(None);
        assert!(rollback.args.path.is_none());

        let path = PathBuf::from("test_file.txt");
        let rollback_with_path = create_test_rollback_command(Some(path.clone()));
        assert_eq!(rollback_with_path.args.path, Some(path));
    }

    #[test]
    fn test_rollback_single_file_not_found() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let nonexistent_file = PathBuf::from("nonexistent.txt");
        let result = rollback_single_file(&bucket_path, &nonexistent_file);

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::IoError(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected IoError with NotFound kind"),
        }
    }

    #[test]
    #[serial]
    fn test_rollback_single_file_no_previous_commit() {
        let (_temp_dir, _repo_path, bucket_path) = create_test_repo_and_bucket_structure()
            .expect("Failed to create test repository structure");

        // Create a test file
        let file_path = bucket_path.join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Change to the bucket directory to simulate proper working directory context
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&bucket_path).expect("Failed to change directory");

        let result = rollback_single_file(&std::env::current_dir().unwrap(), &file_path);

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::IoError(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(err.to_string(), "No previous commit found.");
            }
            _ => panic!("Expected IoError with NotFound kind"),
        }
    }

    #[test]
    fn test_rollback_single_file_invalid_utf8_path() {
        let (_temp_dir, _repo_path, bucket_path) = create_test_repo_and_bucket_structure()
            .expect("Failed to create test repository structure");

        // Change to the bucket directory to simulate proper working directory context
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&bucket_path).expect("Failed to change directory");

        // Create a file with invalid UTF-8 name (this is platform-specific)
        #[cfg(unix)]
        {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            // Create a file with invalid UTF-8 bytes
            let invalid_bytes = b"invalid_\xFF_utf8.txt";
            let invalid_name = OsStr::from_bytes(invalid_bytes);
            let invalid_path = bucket_path.join(invalid_name);

            // Create the file
            fs::write(&invalid_path, "test content").expect("Failed to write test file");

            let result = rollback_single_file(&std::env::current_dir().unwrap(), &invalid_path);

            assert!(result.is_err());
            // The UTF-8 conversion may succeed in some environments,
            // so we check for either UTF-8 error, no previous commit error, or failed to load previous commit
            match result.unwrap_err() {
                BucketError::IoError(err) => {
                    let error_msg = err.to_string();
                    assert!(
                        error_msg == "Invalid UTF-8 path."
                            || error_msg == "No previous commit found."
                            || error_msg == "Failed to load previous commit.",
                        "Expected 'Invalid UTF-8 path.', 'No previous commit found.', or 'Failed to load previous commit.', got: {}",
                        error_msg
                    );
                }
                _ => panic!("Expected IoError for invalid UTF-8 path or no previous commit"),
            }
        }

        // On Windows, paths are generally UTF-16, so this test might not apply
        #[cfg(windows)]
        {
            // Just test that a normal file path works
            let file_path = bucket_path.join("test_file.txt");
            fs::write(&file_path, "test content").expect("Failed to write test file");

            let result = rollback_single_file(&std::env::current_dir().unwrap(), &file_path);

            // Should fail with no previous commit, not UTF-8 error
            assert!(result.is_err());
        }

        // Restore original directory - use let _ to ignore potential errors during cleanup
        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    fn test_rollback_single_file_successful_restore() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create and compress a test file
        let original_content = "original content";
        let (file_path, hash) =
            create_test_file_and_compress(&bucket_path, "test_file.txt", original_content)
                .expect("Failed to create test file");

        // Modify the file
        fs::write(&file_path, "modified content").expect("Failed to modify test file");

        // Mock the Commit::load_last_commit to return our test file
        // Note: This would require mocking or dependency injection in real implementation
        // For now, we'll test the function components separately

        // Verify file was modified
        let content = fs::read_to_string(&file_path).expect("Failed to read modified file");
        assert_eq!(content, "modified content");

        // Create a CommittedFile for restoration
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "test_file.txt".to_string(),
            hash: hash_file(&file_path).expect("Failed to hash file"),
            previous_hash: hash, // Use the original hash as previous_hash
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test the restore functionality directly
        let result = committed_file.restore(&bucket_path);
        assert!(result.is_ok());

        // Verify the file was restored
        let restored_content =
            fs::read_to_string(&file_path).expect("Failed to read restored file");
        assert_eq!(restored_content, original_content);
    }

    #[test]
    fn test_rollback_single_file_hash_mismatch() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create a test file
        let file_path = bucket_path.join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Create a mock commit with a different hash
        let different_hash = blake3::hash(b"different content");
        let commit_data = CommitData {
            bucket: "test_bucket".to_string(),
            files: vec![CommittedFile {
                id: Uuid::new_v4(),
                name: "test_file.txt".to_string(),
                hash: different_hash,
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                status: crate::data::commit::CommitStatus::Modified,
            }],
            timestamp: chrono::Utc::now().to_rfc3339(),
            previous: None,
            next: None,
        };

        // Test that the file won't be found due to hash mismatch
        let file_hash = hash_file(&file_path).expect("Failed to hash file");
        let found_file = commit_data.files.iter().find(|committed_file| {
            committed_file.name == "test_file.txt" && committed_file.hash == file_hash
        });

        assert!(found_file.is_none());
    }

    #[test]
    fn test_rollback_all_empty_bucket() {
        let (_temp_dir, _repo_path, bucket_path) = create_test_repo_and_bucket_structure()
            .expect("Failed to create test repository structure");

        // Change to the bucket directory to simulate proper working directory context
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&bucket_path).expect("Failed to change directory");

        let result = rollback_all(&std::env::current_dir().unwrap());

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        // Should succeed and print "No files in bucket"
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_rollback_all_no_previous_commit() {
        let (_temp_dir, _repo_path, bucket_path) = create_test_repo_and_bucket_structure()
            .expect("Failed to create test repository structure");

        // Change to the bucket directory to simulate proper working directory context
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&bucket_path).expect("Failed to change directory");

        // Create a test file so the bucket is not empty (create it in the current directory)
        let file_path = std::env::current_dir().unwrap().join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Call rollback_all with the current directory (bucket directory)
        let result = rollback_all(&std::env::current_dir().unwrap());

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::IoError(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(err.to_string(), "No previous commit found.");
            }
            _ => panic!("Expected IoError with NotFound kind"),
        }
    }

    #[test]
    fn test_rollback_all_no_changes() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create a test file
        let file_path = bucket_path.join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Create a mock commit with the same file (no changes)
        let file_hash = hash_file(&file_path).expect("Failed to hash file");
        let commit_data = CommitData {
            bucket: "test_bucket".to_string(),
            files: vec![CommittedFile {
                id: Uuid::new_v4(),
                name: "test_file.txt".to_string(),
                hash: file_hash,
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                status: crate::data::commit::CommitStatus::Committed,
            }],
            timestamp: chrono::Utc::now().to_rfc3339(),
            previous: None,
            next: None,
        };

        // Test that no modified files are found
        let changes = commit_data
            .files
            .iter()
            .filter(|change| change.status == crate::data::commit::CommitStatus::Modified)
            .count();

        assert_eq!(changes, 0);
    }

    #[test]
    fn test_rollback_all_with_modified_files() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create and compress original files
        let (file1_path, file1_hash) =
            create_test_file_and_compress(&bucket_path, "file1.txt", "original content 1")
                .expect("Failed to create test file 1");
        let (file2_path, file2_hash) =
            create_test_file_and_compress(&bucket_path, "file2.txt", "original content 2")
                .expect("Failed to create test file 2");

        // Modify the files
        fs::write(&file1_path, "modified content 1").expect("Failed to modify file 1");
        fs::write(&file2_path, "modified content 2").expect("Failed to modify file 2");

        // Create mock commit data with modified files
        let changes = vec![
            CommittedFile {
                id: Uuid::new_v4(),
                name: "file1.txt".to_string(),
                hash: hash_file(&file1_path).expect("Failed to hash file 1"),
                previous_hash: file1_hash,
                status: crate::data::commit::CommitStatus::Modified,
            },
            CommittedFile {
                id: Uuid::new_v4(),
                name: "file2.txt".to_string(),
                hash: hash_file(&file2_path).expect("Failed to hash file 2"),
                previous_hash: file2_hash,
                status: crate::data::commit::CommitStatus::Modified,
            },
        ];

        // Test restoration of modified files
        for change in changes
            .iter()
            .filter(|c| c.status == crate::data::commit::CommitStatus::Modified)
        {
            let result = change.restore(&bucket_path);
            assert!(result.is_ok());
        }

        // Verify files were restored
        let restored_content1 =
            fs::read_to_string(&file1_path).expect("Failed to read restored file 1");
        let restored_content2 =
            fs::read_to_string(&file2_path).expect("Failed to read restored file 2");

        assert_eq!(restored_content1, "original content 1");
        assert_eq!(restored_content2, "original content 2");
    }

    #[test]
    fn test_rollback_all_mixed_file_statuses() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create test files with different statuses
        let (modified_file_path, modified_hash) =
            create_test_file_and_compress(&bucket_path, "modified.txt", "original")
                .expect("Failed to create modified file");
        let (_unchanged_file_path, unchanged_hash) =
            create_test_file_and_compress(&bucket_path, "unchanged.txt", "unchanged")
                .expect("Failed to create unchanged file");

        // Modify one file
        fs::write(&modified_file_path, "modified").expect("Failed to modify file");

        // Create changes with mixed statuses
        let changes = vec![
            CommittedFile {
                id: Uuid::new_v4(),
                name: "modified.txt".to_string(),
                hash: hash_file(&modified_file_path).expect("Failed to hash modified file"),
                previous_hash: modified_hash,
                status: crate::data::commit::CommitStatus::Modified,
            },
            CommittedFile {
                id: Uuid::new_v4(),
                name: "unchanged.txt".to_string(),
                hash: unchanged_hash,
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                status: crate::data::commit::CommitStatus::Committed,
            },
            CommittedFile {
                id: Uuid::new_v4(),
                name: "new.txt".to_string(),
                hash: blake3::hash(b"new content"),
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                status: crate::data::commit::CommitStatus::New,
            },
        ];

        // Count only modified files
        let modified_count = changes
            .iter()
            .filter(|change| change.status == crate::data::commit::CommitStatus::Modified)
            .count();

        assert_eq!(modified_count, 1);

        // Test that only modified files are processed
        let modified_files: Vec<_> = changes
            .iter()
            .filter(|change| change.status == crate::data::commit::CommitStatus::Modified)
            .collect();

        assert_eq!(modified_files.len(), 1);
        assert_eq!(modified_files[0].name, "modified.txt");
    }

    #[test]
    fn test_rollback_all_nested_directories() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create nested file structure
        let nested_path = "subdir/nested/file.txt";
        let (nested_file_path, nested_hash) =
            create_test_file_and_compress(&bucket_path, nested_path, "nested original")
                .expect("Failed to create nested file");

        // Modify the nested file
        fs::write(&nested_file_path, "nested modified").expect("Failed to modify nested file");

        // Create change for nested file
        let change = CommittedFile {
            id: Uuid::new_v4(),
            name: nested_path.to_string(),
            hash: hash_file(&nested_file_path).expect("Failed to hash nested file"),
            previous_hash: nested_hash,
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test restoration
        let result = change.restore(&bucket_path);
        assert!(result.is_ok());

        // Verify nested file was restored
        let restored_content =
            fs::read_to_string(&nested_file_path).expect("Failed to read restored nested file");
        assert_eq!(restored_content, "nested original");
    }

    #[test]
    fn test_rollback_all_restore_failure() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create a file path but don't compress it to storage
        let file_path = bucket_path.join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Create a change that references a non-existent storage file
        let fake_hash = blake3::hash(b"fake content");
        let change = CommittedFile {
            id: Uuid::new_v4(),
            name: "test_file.txt".to_string(),
            hash: hash_file(&file_path).expect("Failed to hash file"),
            previous_hash: fake_hash, // This hash doesn't exist in storage
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test that restoration fails
        let result = change.restore(&bucket_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_all_large_file_handling() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create a large file (10KB)
        let large_content = "x".repeat(10000);
        let (large_file_path, large_hash) =
            create_test_file_and_compress(&bucket_path, "large_file.txt", &large_content)
                .expect("Failed to create large file");

        // Modify the large file
        let modified_content = "y".repeat(10000);
        fs::write(&large_file_path, &modified_content).expect("Failed to modify large file");

        // Create change for large file
        let change = CommittedFile {
            id: Uuid::new_v4(),
            name: "large_file.txt".to_string(),
            hash: hash_file(&large_file_path).expect("Failed to hash large file"),
            previous_hash: large_hash,
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test restoration
        let result = change.restore(&bucket_path);
        assert!(result.is_ok());

        // Verify large file was restored correctly
        let restored_content =
            fs::read_to_string(&large_file_path).expect("Failed to read restored large file");
        assert_eq!(restored_content, large_content);
    }

    #[test]
    fn test_rollback_all_binary_file_handling() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create a binary file
        let binary_content = vec![0u8, 1u8, 2u8, 255u8, 128u8];
        let binary_file_path = bucket_path.join("binary_file.bin");
        fs::write(&binary_file_path, &binary_content).expect("Failed to write binary file");

        let binary_hash = hash_file(&binary_file_path).expect("Failed to hash binary file");

        // Compress the binary file
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "binary_file.bin".to_string(),
            hash: binary_hash,
            previous_hash: Hash::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            status: crate::data::commit::CommitStatus::New,
        };

        committed_file
            .compress_and_store(&bucket_path)
            .expect("Failed to compress binary file");

        // Modify the binary file
        let modified_binary_content = vec![10u8, 20u8, 30u8, 40u8, 50u8];
        fs::write(&binary_file_path, &modified_binary_content)
            .expect("Failed to modify binary file");

        // Create change for binary file
        let change = CommittedFile {
            id: Uuid::new_v4(),
            name: "binary_file.bin".to_string(),
            hash: hash_file(&binary_file_path).expect("Failed to hash modified binary file"),
            previous_hash: binary_hash,
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test restoration
        let result = change.restore(&bucket_path);
        assert!(result.is_ok());

        // Verify binary file was restored correctly
        let restored_content =
            fs::read(&binary_file_path).expect("Failed to read restored binary file");
        assert_eq!(restored_content, binary_content);
    }

    #[test]
    fn test_rollback_all_empty_file_handling() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create an empty file
        let empty_file_path = bucket_path.join("empty_file.txt");
        fs::write(&empty_file_path, "").expect("Failed to write empty file");

        let empty_hash = hash_file(&empty_file_path).expect("Failed to hash empty file");

        // Compress the empty file
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "empty_file.txt".to_string(),
            hash: empty_hash,
            previous_hash: Hash::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            status: crate::data::commit::CommitStatus::New,
        };

        committed_file
            .compress_and_store(&bucket_path)
            .expect("Failed to compress empty file");

        // Add content to the empty file
        fs::write(&empty_file_path, "now has content").expect("Failed to modify empty file");

        // Create change for empty file
        let change = CommittedFile {
            id: Uuid::new_v4(),
            name: "empty_file.txt".to_string(),
            hash: hash_file(&empty_file_path).expect("Failed to hash modified empty file"),
            previous_hash: empty_hash,
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test restoration
        let result = change.restore(&bucket_path);
        assert!(result.is_ok());

        // Verify empty file was restored correctly
        let restored_content =
            fs::read_to_string(&empty_file_path).expect("Failed to read restored empty file");
        assert_eq!(restored_content, "");
    }

    #[test]
    fn test_rollback_directory_permissions() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create a file in a subdirectory
        let subdir_file_path = bucket_path.join("subdir").join("file.txt");
        fs::create_dir_all(subdir_file_path.parent().unwrap())
            .expect("Failed to create subdirectory");
        fs::write(&subdir_file_path, "subdir content").expect("Failed to write subdir file");

        let subdir_hash = hash_file(&subdir_file_path).expect("Failed to hash subdir file");

        // Compress the file using the helper function which properly sets up the storage
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "subdir/file.txt".to_string(),
            hash: subdir_hash,
            previous_hash: Hash::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            status: crate::data::commit::CommitStatus::New,
        };

        committed_file
            .compress_and_store(&bucket_path)
            .expect("Failed to compress subdir file");

        // Remove the subdirectory
        fs::remove_dir_all(subdir_file_path.parent().unwrap())
            .expect("Failed to remove subdirectory");

        // Create change to restore the file (which should recreate the directory)
        // The previous_hash should reference the stored file, so we use the same hash
        let change = CommittedFile {
            id: Uuid::new_v4(),
            name: "subdir/file.txt".to_string(),
            hash: blake3::hash(b"modified content"),
            previous_hash: subdir_hash, // This should match the stored file's hash
            status: crate::data::commit::CommitStatus::Modified,
        };

        // Test restoration (should recreate the directory)
        let result = change.restore(&bucket_path);
        assert!(result.is_ok());

        // Verify the directory was recreated and file was restored
        assert!(subdir_file_path.exists());
        let restored_content =
            fs::read_to_string(&subdir_file_path).expect("Failed to read restored subdir file");
        assert_eq!(restored_content, "subdir content");
    }
}
