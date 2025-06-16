mod common;
#[cfg(test)]
mod tests {
    use duckdb::Connection;
    use predicates::prelude::*;
    use uuid::{Uuid, Version};
    use crate::common::tests::get_test_dir;
    use serial_test::serial;

    /// Test the `create` command.
    ///
    /// # Commands
    /// `$ buckets create test_repo`
    ///
    /// # Expected output
    ///
    #[test]
    #[serial]
    fn test_cli_create_no_repo() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .stderr(predicate::str::contains("Not in a buckets repository"))
            .failure();
    }

    #[test]
    #[serial]
    fn test_cli_create() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success()
            .stdout(predicate::str::contains(""))
            .stderr(predicate::str::is_empty());

        let repo_dir = temp_dir.as_path().join("test_repo");
        assert!(repo_dir.exists());
        assert!(repo_dir.is_dir());

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success()
            .stdout(predicate::str::contains(""))
            .stderr(predicate::str::is_empty());

        let bucket_path = repo_dir.join("test_bucket").join(".b");
        assert!(bucket_path.exists());
        assert!(bucket_path.join("storage").exists());

        // Check if added to database
        let db_path = repo_dir.join(".buckets").join("buckets.db");
        let connection = Connection::open(db_path).expect("Failed to open database");

        match connection.prepare("SELECT * FROM buckets WHERE name = 'test_bucket'") {
            Ok(mut statement) => {
                // Execute the query and fetch rows
                let rows = statement.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?, // Assuming column 0 is a string
                        row.get::<_, String>(1)?, // Adjust based on your schema
                    ))
                });

                match rows {
                    Ok(rows) => {
                        for row in rows {
                            match row {
                                Ok((id, name)) => {
                                    match Uuid::parse_str(&id) {
                                        Ok(uuid) => {
                                            // Check if UUID is version 4
                                            assert_eq!(uuid.get_version(), Some(Version::Random) );
                                        }
                                        Err(e) => {
                                            println!("Invalid UUID: {}. Error: {}", id, e);
                                        }
                                    }
                                    assert_eq!(name, "test_bucket")
                                },
                                Err(e) => eprintln!("Error retrieving row: {}", e),
                            }
                        }
                    }
                    Err(e) => eprintln!("Error querying rows: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Error preparing query: {}", e);
            }
        }
    }

    /// Test creating bucket that already exists (should fail)
    #[test]
    #[serial]
    fn test_cli_create_bucket_already_exists() {
        let temp_dir = get_test_dir();
        
        // Initialize repository
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.as_path().join("test_repo");
        
        // Create bucket first time (should succeed)
        let mut cmd2 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd2.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("duplicate_bucket")
            .assert()
            .success();
        
        // Try to create same bucket again (should fail)
        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd3.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("duplicate_bucket")
            .assert()
            .failure()
            .stderr(predicate::str::contains("already exists").or(predicate::str::contains("duplicate")));
    }

    /// Test creating bucket with invalid name characters
    #[test]
    #[serial]
    fn test_cli_create_invalid_bucket_name() {
        let temp_dir = get_test_dir();
        
        // Initialize repository
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.as_path().join("test_repo");
        
        // Test various invalid bucket names
        let invalid_names = vec![
            ".", // current directory
            "..", // parent directory
            "bucket/with/slashes", // path separators
            // Note: null characters can't be tested via command line
        ];
        
        for invalid_name in invalid_names {
            let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
            cmd.current_dir(repo_dir.as_path())
                .arg("create")
                .arg(invalid_name)
                .assert()
                .failure();
        }
    }

    /// Test creating bucket with very long name
    #[test]
    #[serial]
    fn test_cli_create_long_bucket_name() {
        let temp_dir = get_test_dir();
        
        // Initialize repository
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.as_path().join("test_repo");
        
        // Create very long bucket name (255+ characters)
        let long_name = "a".repeat(300);
        
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(repo_dir.as_path())
            .arg("create")
            .arg(&long_name)
            .assert()
            .failure(); // May fail due to filesystem limitations
    }

    /// Test creating bucket without providing name argument
    #[test]
    #[serial]
    fn test_cli_create_missing_name() {
        let temp_dir = get_test_dir();
        
        // Initialize repository
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.as_path().join("test_repo");
        
        // Try to create bucket without name
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(repo_dir.as_path())
            .arg("create")
            .assert()
            .failure(); // Should fail due to missing required argument
    }

    /// Test creating bucket with special characters (should succeed)
    #[test]
    #[serial]
    fn test_cli_create_special_characters() {
        let temp_dir = get_test_dir();
        
        // Initialize repository
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.as_path().join("test_repo");
        
        // Test bucket names with special characters that should be valid
        let valid_special_names = vec![
            "bucket-with-dashes",
            "bucket_with_underscores", 
            "bucket.with.dots",
            "bucket123",
            "123bucket",
        ];
        
        for name in valid_special_names {
            let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
            cmd.current_dir(repo_dir.as_path())
                .arg("create")
                .arg(name)
                .assert()
                .success();
        }
    }
}