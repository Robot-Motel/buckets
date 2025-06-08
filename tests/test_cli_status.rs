mod common;
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use predicates::prelude::predicate;
    use crate::common::tests::get_test_dir;
    use serial_test::serial;

    /// Test the `status` command.
    ///
    /// # Commands
    /// `$ buckets status`
    ///
    /// # Expected output
    ///
    #[test]
    #[serial]
    fn test_cli_status() {
        let repo_dir = setup();

        let bucket_dir = repo_dir.join("test_bucket");
        let file_path = bucket_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"test").expect("Failed to write to file");

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(repo_dir.as_path())
            .arg("status")
            .assert()
            .stdout(predicate::str::contains("Number of buckets: 1"))
            .success();

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(bucket_dir.as_path())
            .arg("status")
            .assert()
            .stdout(predicate::str::contains("new:    test_file.txt"))
            .success();

        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(bucket_dir.as_path())
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