mod common;
#[cfg(test)]
mod tests {
    use duckdb::Connection;
    use predicates::prelude::predicate;
    use uuid::{Uuid, Version};
    use crate::common::tests::get_test_dir;

    /// Test the `create` command.
    ///
    /// # Commands
    /// `$ buckets create test_repo`
    ///
    /// # Expected output
    ///
    #[test]
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
}