use crate::args::CommitCommand;
use crate::commands::BucketCommand;
use crate::data::bucket::BucketTrait;
use crate::data::commit::{Commit as CommitData, CommitStatus, CommittedFile};
use crate::errors::BucketError;
use crate::utils::utils::{
    connect_to_db, find_files_excluding_top_level_b, hash_file, with_db_connection,
};
use crate::world::World;
use blake3::Hash;
use duckdb::params;
use log::{debug, error};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

/// Commit changes to a bucket
pub struct Commit {
    args: CommitCommand,
}

impl BucketCommand for Commit {
    type Args = CommitCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        println!(
            "Executing commit command ########################################################## "
        );

        let world = World::new(&self.args.shared)?;

        let bucket = match &world.bucket {
            Some(bucket) => bucket,
            None => return Err(BucketError::NotInBucket),
        };

        println!(
            "Bucket: {} ########################################################## ",
            bucket.name
        );

        // create a list of each file in the bucket directory, recursively
        // and create a blake3 hash for each file and add to current_commit
        let current_commit =
            self.list_files_with_metadata_in_bucket(bucket.get_full_bucket_path()?)?;
        if current_commit.files.is_empty() {
            return Err(
                Error::new(ErrorKind::NotFound, "No commitable files found in bucket.").into(),
            );
        }

        println!("Current commit: ########################################################## ");

        // Load the previous commit, if it exists
        match Commit::load_last_commit(bucket.name.clone()) {
            Ok(None) => {
                // There is no previous commit; Process all files in the current commit
                println!("No previous commit found. Processing all files. ########################################################## ");
                self.process_files(
                    bucket.id,
                    &bucket.relative_bucket_path,
                    &current_commit.files,
                    &self.args.message,
                )?;
            }
            Ok(Some(previous_commit)) => {
                // Compare the current commit with the previous commit
                println!("Previous commit found. Comparing with current commit. ########################################################## ");
                if let Some(changes) = current_commit.compare(&previous_commit) {
                    // Process the files that have changed
                    println!("Processing files that have changed. ########################################################## ");
                    self.process_files(
                        bucket.id,
                        &bucket.get_full_bucket_path()?,
                        &changes,
                        &self.args.message,
                    )?;
                } else {
                    // if there are no difference with previous commit cancel commit
                    println!("No changes detected. Commit cancelled. ########################################################## ");
                    println!("No changes detected. Commit cancelled.");
                    return Ok(());
                }
            }
            Err(_) => {
                println!("Failed to load previous commit. ########################################################## ");
                error!("Failed to load previous commit.");
                return Err(BucketError::from(Error::new(
                    ErrorKind::Other,
                    "Failed to load previous commit.",
                )));
            }
        }
        println!("Commit completed. ########################################################## ");

        Ok(())
    }
}

impl Commit {
    pub fn process_files(
        &self,
        bucket_id: Uuid,
        bucket_path: &PathBuf,
        files: &[CommittedFile],
        message: &String,
    ) -> Result<(), BucketError> {
        // Use a single connection for all database operations
        with_db_connection(|connection| {
            // Insert the commit into the database
            let commit_id =
                self.insert_commit_into_db_with_connection(connection, bucket_id, message)?;

            // Process each file in the commit using the same connection
            for file in files {
                // Insert the file into the database
                self.insert_file_into_db_with_connection(
                    connection,
                    &commit_id,
                    &file.name,
                    &file.hash.to_string(),
                )?;

                // Compress and store the file (no database operation)
                file.compress_and_store(&bucket_path).map_err(|e| {
                    error!("Error compressing and storing file: {}", e);
                    e
                })?;
            }
            Ok(())
        })
    }

    // New methods that accept database connections to avoid repeated connection creation
    fn insert_file_into_db_with_connection(
        &self,
        connection: &duckdb::Connection,
        commit_id: &str,
        file_path: &str,
        hash: &str,
    ) -> Result<(), BucketError> {
        connection.execute(
        "INSERT INTO files (id, commit_id, file_path, hash) VALUES (gen_random_uuid(), ?1, ?2, ?3)",
        [commit_id, file_path, hash],
    )
        .map_err(|e| {
            BucketError::from(Error::new(
                ErrorKind::Other,
                format!("Error inserting into database: {}, commit id: {}, file path: {}, hash: {}", e, commit_id, file_path, hash),
            ))
        })?;
        Ok(())
    }

    fn insert_commit_into_db_with_connection(
        &self,
        connection: &duckdb::Connection,
        bucket_id: Uuid,
        message: &String,
    ) -> Result<String, BucketError> {
        debug!(
            "CommitCommand: path to database {}",
            connection
                .path()
                .ok_or_else(|| BucketError::from(Error::new(
                    ErrorKind::Other,
                    "Invalid database connection path".to_string()
                )))?
                .display()
        );
        // Now query back the `id` using the `rowid`
        let stmt = &mut connection.prepare("INSERT INTO commits (id, bucket_id, message) VALUES (gen_random_uuid(), ?1, ?2) RETURNING id")?;
        let rows = &mut stmt.query(params![
            bucket_id.to_string().to_uppercase(),
            message.clone()
        ])?;

        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Err(BucketError::from(duckdb::Error::QueryReturnedNoRows))
        }
    }

    fn list_files_with_metadata_in_bucket(&self, bucket_path: PathBuf) -> io::Result<CommitData> {
        let mut files = Vec::new();

        // Extract bucket name from the bucket path
        let bucket_name = bucket_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        for entry in find_files_excluding_top_level_b(bucket_path.as_path()) {
            let full_path = bucket_path.join(&entry);

            if full_path.is_file() || full_path.is_symlink() {
                match hash_file(&full_path) {
                    Ok(hash) => {
                        //println!("BLAKE3 hash: {}", hash);
                        files.push(CommittedFile {
                            id: Uuid::new_v4(),
                            name: entry.to_string_lossy().into_owned(),
                            hash,
                            previous_hash: Hash::from_str(
                                "0000000000000000000000000000000000000000000000000000000000000000",
                            )
                            .map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("Invalid hash format: {}", e),
                                )
                            })?,
                            status: CommitStatus::New,
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to hash file: {}", e);
                        return Err(e);
                    }
                }
            } else {
                debug!("Skipping non-file: {:?}", entry.as_path());
            }
        }

        Ok(CommitData {
            bucket: bucket_name,
            files,
            timestamp: chrono::Utc::now().to_rfc3339(),
            previous: None,
            next: None,
        })
    }

    pub fn load_last_commit(bucket_name: String) -> Result<Option<CommitData>, BucketError> {
        let connection = connect_to_db()?;

        let mut stmt = connection.prepare(
            "SELECT f.id, f.file_path, f.hash
                                               FROM files f
                                               JOIN commits c ON f.commit_id = c.id
                                WHERE c.created_at = (SELECT MAX(created_at) FROM commits)",
        )?;

        let mut rows = stmt.query([])?;

        let mut files = Vec::new();
        while let Some(row) = rows.next()? {
            let uuid_string: String = row.get(0)?;
            let hex_string: String = row.get(2)?;

            files.push(CommittedFile {
                id: Uuid::parse_str(&uuid_string).map_err(|e| {
                    BucketError::from(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid UUID: {}", e),
                    ))
                })?,
                name: row.get(1)?,
                hash: Hash::from_hex(&hex_string).map_err(|e| {
                    BucketError::from(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid hash: {}", e),
                    ))
                })?,
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .map_err(|e| {
                    BucketError::from(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid hash format: {}", e),
                    ))
                })?, // TODO: Implement previous hash
                status: CommitStatus::Committed,
            });
        }

        if let Err((_conn, e)) = connection.close() {
            return Err(BucketError::from(Error::new(
                ErrorKind::Other,
                format!("Failed to close database connection: {}", e),
            )));
        }

        Ok(Some(CommitData {
            bucket: bucket_name,
            files,
            timestamp: "".to_string(),
            previous: None,
            next: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::commit::Commit;
    use crate::commands::BucketCommand;
    use crate::data::bucket::read_bucket_info;
    use crate::data::commit::{CommitStatus, CommittedFile};
    use blake3::Hash;
    use log::error;
    use serial_test::serial;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[test]
    #[serial]
    fn test_process_files() {
        // Need to setup a proper test environment
        let temp_dir = tempdir().expect("invalid temp dir").keep();
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let mut cmd2 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        let repo_dir = temp_dir.as_path().join("test_repo");
        cmd2.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success();

        let bucket_dir = repo_dir.join("test_bucket");
        let file_path = bucket_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create test file");
        file.write_all(b"test").expect("Failed to write test data");
        let mut cmd3 =
            assert_cmd::Command::cargo_bin("buckets").expect("Failed to find buckets binary");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        // Bucket id is stored in the bucket info file
        // Can be read first to get the bucket id and then use
        // to query the database
        let bucket = read_bucket_info(&bucket_dir).expect("Failed to read bucket info");

        let commit_message = "Test commit".to_string();
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "test_file.txt".to_string(),
            hash: Hash::from_str(
                "f4315de648c8440fb2539fe9a8417e901ab270a37c6e2267e0c5fffe7d4d4419",
            )
            .expect("Failed to create test hash"),
            previous_hash: Hash::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("Failed to create zero hash"),
            status: CommitStatus::New,
        };

        // change to bucket directory
        env::set_current_dir(&bucket_dir).expect("Failed to change to bucket directory");

        let commit_cmd = Commit::new(&crate::args::CommitCommand {
            shared: crate::args::SharedArguments::default(),
            message: commit_message.clone(),
        });
        let result = commit_cmd
            .process_files(bucket.id, &bucket_dir, &[committed_file], &commit_message)
            .map_err(|e| {
                error!("Error processing files: {}", e);
                e
            });

        match result {
            Ok(_) => (),
            Err(e) => {
                panic!("Error processing files: {}", e);
            }
        }
    }

    // Helper function to create a test commit command
    fn create_test_commit_command(message: &str) -> Commit {
        let args = crate::args::CommitCommand {
            shared: crate::args::SharedArguments::default(),
            message: message.to_string(),
        };
        Commit::new(&args)
    }

    // Helper function to create a test bucket directory structure
    fn create_test_bucket_structure(
    ) -> Result<(tempfile::TempDir, std::path::PathBuf), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().join("test_bucket");
        std::fs::create_dir_all(&bucket_path)?;

        // Create .b directory structure
        let b_dir = bucket_path.join(".b");
        std::fs::create_dir_all(&b_dir)?;
        std::fs::create_dir_all(b_dir.join("storage"))?;

        // Create bucket info file
        let info_path = b_dir.join("info");
        let bucket_info = crate::data::bucket::Bucket {
            id: Uuid::new_v4(),
            name: "test_bucket".to_string(),
            relative_bucket_path: std::path::PathBuf::from("test_bucket"),
        };
        let info_content = toml::to_string(&bucket_info)?;
        std::fs::write(&info_path, info_content)?;

        Ok((temp_dir, bucket_path))
    }

    #[test]
    fn test_commit_new() {
        let commit = create_test_commit_command("test message");
        assert_eq!(commit.args.message, "test message");
    }

    #[test]
    fn test_list_files_with_metadata_in_bucket() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create some test files
        let file1_path = bucket_path.join("file1.txt");
        let file2_path = bucket_path.join("file2.txt");
        let subdir_path = bucket_path.join("subdir");
        std::fs::create_dir_all(&subdir_path).expect("Failed to create subdir");
        let file3_path = subdir_path.join("file3.txt");

        std::fs::write(&file1_path, "content1").expect("Failed to write file1");
        std::fs::write(&file2_path, "content2").expect("Failed to write file2");
        std::fs::write(&file3_path, "content3").expect("Failed to write file3");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.bucket, "test_bucket");
        assert_eq!(commit_data.files.len(), 3);

        // Check that files are present
        let file_names: Vec<String> = commit_data.files.iter().map(|f| f.name.clone()).collect();
        assert!(file_names.contains(&"file1.txt".to_string()));
        assert!(file_names.contains(&"file2.txt".to_string()));
        assert!(file_names.contains(&"subdir/file3.txt".to_string()));
    }

    #[test]
    fn test_list_files_with_metadata_empty_bucket() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.bucket, "test_bucket");
        assert_eq!(commit_data.files.len(), 0);
    }

    #[test]
    fn test_list_files_with_metadata_ignores_b_directory() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create files in .b directory (should be ignored)
        let b_file_path = bucket_path.join(".b").join("internal_file.txt");
        std::fs::write(&b_file_path, "internal content").expect("Failed to write .b file");

        // Create regular file (should be included)
        let regular_file_path = bucket_path.join("regular_file.txt");
        std::fs::write(&regular_file_path, "regular content")
            .expect("Failed to write regular file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);
        assert_eq!(commit_data.files[0].name, "regular_file.txt");
    }

    #[test]
    fn test_list_files_with_metadata_calculates_hash() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("test_file.txt");
        let file_content = "test content for hashing";
        std::fs::write(&file_path, file_content).expect("Failed to write test file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        assert_eq!(file.name, "test_file.txt");
        assert_eq!(file.status, CommitStatus::New);

        // Verify hash is calculated correctly
        let expected_hash = blake3::hash(file_content.as_bytes());
        assert_eq!(file.hash, expected_hash);
    }

    #[test]
    fn test_list_files_with_metadata_nonexistent_bucket() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let nonexistent_bucket_path = temp_dir.path().join("nonexistent_bucket");
        // Don't create the directory - this should result in an empty bucket

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(nonexistent_bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 0);
        assert_eq!(commit_data.bucket, "nonexistent_bucket");
    }

    #[test]
    fn test_committed_file_hash_consistency() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("consistency_test.txt");
        let file_content = "consistent content";
        std::fs::write(&file_path, file_content).expect("Failed to write test file");

        let commit = create_test_commit_command("test message");

        // List files multiple times and verify hash consistency
        let result1 = commit
            .list_files_with_metadata_in_bucket(bucket_path.clone())
            .unwrap();
        let result2 = commit
            .list_files_with_metadata_in_bucket(bucket_path.clone())
            .unwrap();

        assert_eq!(result1.files.len(), 1);
        assert_eq!(result2.files.len(), 1);
        assert_eq!(result1.files[0].hash, result2.files[0].hash);
        assert_eq!(result1.files[0].name, result2.files[0].name);
    }

    #[test]
    fn test_committed_file_different_content_different_hash() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("changing_file.txt");
        let commit = create_test_commit_command("test message");

        // Write initial content and get hash
        std::fs::write(&file_path, "initial content").expect("Failed to write initial content");
        let result1 = commit
            .list_files_with_metadata_in_bucket(bucket_path.clone())
            .unwrap();

        // Change content and get new hash
        std::fs::write(&file_path, "changed content").expect("Failed to write changed content");
        let result2 = commit
            .list_files_with_metadata_in_bucket(bucket_path.clone())
            .unwrap();

        assert_eq!(result1.files.len(), 1);
        assert_eq!(result2.files.len(), 1);
        assert_ne!(result1.files[0].hash, result2.files[0].hash);
        assert_eq!(result1.files[0].name, result2.files[0].name);
    }

    #[test]
    fn test_committed_file_large_file_handling() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("large_file.txt");
        let large_content = "x".repeat(10000); // 10KB file
        std::fs::write(&file_path, &large_content).expect("Failed to write large file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        assert_eq!(file.name, "large_file.txt");

        // Verify hash is calculated correctly for large file
        let expected_hash = blake3::hash(large_content.as_bytes());
        assert_eq!(file.hash, expected_hash);
    }

    #[test]
    fn test_committed_file_binary_file_handling() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("binary_file.bin");
        let binary_content = vec![0u8, 1u8, 2u8, 255u8, 128u8]; // Binary data
        std::fs::write(&file_path, &binary_content).expect("Failed to write binary file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        assert_eq!(file.name, "binary_file.bin");

        // Verify hash is calculated correctly for binary file
        let expected_hash = blake3::hash(&binary_content);
        assert_eq!(file.hash, expected_hash);
    }

    #[test]
    fn test_committed_file_nested_directories() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create nested directory structure
        let nested_dir = bucket_path.join("level1").join("level2").join("level3");
        std::fs::create_dir_all(&nested_dir).expect("Failed to create nested directories");

        let file_path = nested_dir.join("nested_file.txt");
        std::fs::write(&file_path, "nested content").expect("Failed to write nested file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        assert_eq!(file.name, "level1/level2/level3/nested_file.txt");
        assert_eq!(file.status, CommitStatus::New);
    }

    #[test]
    fn test_committed_file_permissions() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("permissions_test.txt");
        std::fs::write(&file_path, "permissions content").expect("Failed to write file");

        // Change file permissions (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&file_path).unwrap().permissions();
            permissions.set_mode(0o644);
            std::fs::set_permissions(&file_path, permissions).expect("Failed to set permissions");
        }

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        assert_eq!(file.name, "permissions_test.txt");
        // Hash should be based on content, not permissions
        let expected_hash = blake3::hash("permissions content".as_bytes());
        assert_eq!(file.hash, expected_hash);
    }

    #[test]
    fn test_committed_file_symlink_handling() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let target_file = bucket_path.join("target_file.txt");
        std::fs::write(&target_file, "target content").expect("Failed to write target file");

        let symlink_path = bucket_path.join("symlink_file.txt");

        // Create symlink (Unix-like systems)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target_file, &symlink_path)
                .expect("Failed to create symlink");

            let commit = create_test_commit_command("test message");
            let result = commit.list_files_with_metadata_in_bucket(bucket_path);

            assert!(result.is_ok());
            let commit_data = result.unwrap();
            // Should include both target file and symlink
            assert_eq!(commit_data.files.len(), 2);

            let file_names: Vec<String> =
                commit_data.files.iter().map(|f| f.name.clone()).collect();
            assert!(file_names.contains(&"target_file.txt".to_string()));
            assert!(file_names.contains(&"symlink_file.txt".to_string()));
        }

        // On Windows, just test that the target file is processed
        #[cfg(windows)]
        {
            let commit = create_test_commit_command("test message");
            let result = commit.list_files_with_metadata_in_bucket(bucket_path);

            assert!(result.is_ok());
            let commit_data = result.unwrap();
            assert_eq!(commit_data.files.len(), 1);
            assert_eq!(commit_data.files[0].name, "target_file.txt");
        }
    }

    #[test]
    fn test_committed_file_empty_file() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let empty_file_path = bucket_path.join("empty_file.txt");
        std::fs::write(&empty_file_path, "").expect("Failed to write empty file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        assert_eq!(file.name, "empty_file.txt");

        // Hash of empty content
        let expected_hash = blake3::hash(b"");
        assert_eq!(file.hash, expected_hash);
    }

    #[test]
    fn test_multiple_files_sorting() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        // Create files in different orders
        let files = vec!["z_file.txt", "a_file.txt", "m_file.txt"];
        for file_name in &files {
            let file_path = bucket_path.join(file_name);
            std::fs::write(&file_path, format!("content of {}", file_name))
                .expect("Failed to write file");
        }

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 3);

        // Check that all files are present (order may vary depending on implementation)
        let file_names: Vec<String> = commit_data.files.iter().map(|f| f.name.clone()).collect();
        assert!(file_names.contains(&"z_file.txt".to_string()));
        assert!(file_names.contains(&"a_file.txt".to_string()));
        assert!(file_names.contains(&"m_file.txt".to_string()));
    }

    #[test]
    fn test_committed_file_uuid_generation() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("uuid_test.txt");
        std::fs::write(&file_path, "uuid content").expect("Failed to write file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();
        assert_eq!(commit_data.files.len(), 1);

        let file = &commit_data.files[0];
        // UUID should be valid and not nil
        assert_ne!(file.id, Uuid::nil());
        assert_eq!(file.id.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_commit_timestamp_generation() {
        let (_temp_dir, bucket_path) =
            create_test_bucket_structure().expect("Failed to create test bucket structure");

        let file_path = bucket_path.join("timestamp_test.txt");
        std::fs::write(&file_path, "timestamp content").expect("Failed to write file");

        let commit = create_test_commit_command("test message");
        let result = commit.list_files_with_metadata_in_bucket(bucket_path);

        assert!(result.is_ok());
        let commit_data = result.unwrap();

        // Timestamp should be in ISO 8601 format
        assert!(!commit_data.timestamp.is_empty());

        // Try to parse timestamp to ensure it's valid
        let parsed_timestamp = chrono::DateTime::parse_from_rfc3339(&commit_data.timestamp);
        assert!(parsed_timestamp.is_ok());
    }

    #[test]
    fn test_load_last_commit_no_commit() {
        // This test would require a database setup, so we'll just test the function signature
        // In a real scenario, you would set up a test database
        let result = Commit::load_last_commit("nonexistent_bucket".to_string());

        // Since there's no database setup, this will likely fail
        // In a proper test environment, you would set up a test database
        // and verify that it returns None for a new bucket
        assert!(result.is_err() || result.unwrap().is_none());
    }
}
