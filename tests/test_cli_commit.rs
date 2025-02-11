mod common;
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use duckdb::Connection;
    use uuid::{Uuid, Version};
    use crate::common::tests::get_test_dir;

    /// Test the `commit` command.
    ///
    /// # Commands
    /// `$ buckets commit`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_commit() {
        let repo_dir = setup();

        let bucket_dir = repo_dir.join("test_bucket");
        let file_path = bucket_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"test").expect("Failed to write to file");
        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        // Check if added to database
        let db_path = repo_dir.join(".buckets").join("buckets.db");
        let connection = Connection::open(db_path).expect("Failed to open database");

        match connection.prepare("SELECT * FROM commits WHERE message = 'test message'") {
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
                                Ok((id, bucket_id)) => {
                                    match Uuid::parse_str(&id) {
                                        Ok(uuid) => {
                                            // Check if UUID is version 4
                                            assert_eq!(uuid.get_version(), Some(Version::Random) );
                                        }
                                        Err(e) => {
                                            println!("Invalid UUID: {}. Error: {}", id, e);
                                        }
                                    }
                                    match Uuid::parse_str(&bucket_id) {
                                        Ok(uuid) => {
                                            // Check if UUID is version 4
                                            assert_eq!(uuid.get_version(), Some(Version::Random) );
                                        }
                                        Err(e) => {
                                            println!("Invalid UUID for bucket id: {}. Error: {}", id, e);
                                        }
                                    }
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

    fn setup() -> PathBuf {
        let temp_dir = get_test_dir();
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let mut cmd2 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        let repo_dir = temp_dir.as_path().join("test_repo");
        cmd2.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success();
        repo_dir
    }
}