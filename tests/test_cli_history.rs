mod common;

#[cfg(test)]
mod acceptance_tests {
    use crate::common::tests::get_test_dir;
    use std::fs::File;
    use std::io::Write;
    use predicates::prelude::*;

    /// Test the `history` command.
    ///
    /// # Commands
    /// `$ buckets history`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_history() {
        // Setup repo with a commit
        let repo_dir = setup();
        let bucket_dir = repo_dir.join("test_bucket");
        
        // Create and commit a file
        let file_path = bucket_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"test content").expect("Failed to write to file");

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&bucket_dir)
            .arg("commit")
            .arg("test commit message")
            .assert()
            .success();

        // Test history command
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&bucket_dir)
            .arg("history")
            .assert()
            .success()
            .stdout(predicate::str::contains("test commit message"))
            .stdout(predicate::str::contains("test_bucket"));
    }

    fn setup() -> std::path::PathBuf {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&temp_dir)
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.join("test_repo");
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&repo_dir)
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success();

        repo_dir
    }
}