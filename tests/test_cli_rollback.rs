mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;
    use predicates::prelude::predicate;
    use serial_test::serial;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    /// Test the `rollback` command.
    ///
    /// # Commands
    /// `$ buckets rollback`
    ///
    /// # Expected output
    ///
    #[test]
    #[serial]
    fn test_cli_rollback() {
        let repo_dir = setup();
        let bucket_dir = repo_dir.join("test_bucket");

        let file_path = bucket_dir.join("test_file.txt");
        
        // Create and write initial content
        {
            let mut file_1 = File::create(&file_path).expect("Failed to create file");
            file_1
                .write_all(b"test file 1")
                .expect("Failed to write to file");
        }

        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd1.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        // Modify the file after the commit
        {
            let mut file_1 = File::create(&file_path).expect("Failed to create file for modification");
            file_1
                .write_all(b"change file 1")
                .expect("Failed to write to file");
        }

        let mut cmd2 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd2.current_dir(bucket_dir.as_path())
            .arg("status")
            .assert()
            .stdout(predicate::str::contains("modified:    test_file.txt"))
            .success();

        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("rollback")
            .assert()
            .success();

        let mut cmd4 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd4.current_dir(bucket_dir.as_path())
            .arg("status")
            .assert()
            .stdout(predicate::str::contains("committed:    test_file.txt"))
            .success();
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
