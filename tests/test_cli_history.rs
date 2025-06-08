mod common;

#[cfg(test)]
mod acceptance_tests {
    use crate::common::tests::get_test_dir;
    use serial_test::serial;
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
    #[serial]
    fn test_cli_history_one_commit() {
        // Setup repo with a commit
        let repo_dir = setup();
        let bucket_dir = repo_dir.join("test_bucket");
        
        create_test_file(&bucket_dir, "test_file.txt", "test content");

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

    #[test]
    #[serial]
    fn test_cli_history_multiple_commits() {
        // Setup repo with multiple commits
        let repo_dir = setup();
        let bucket_dir = repo_dir.join("test_bucket");

        create_test_file(&bucket_dir, "test_file.txt", "test content");
        create_test_file(&bucket_dir, "test_file2.txt", "test content 2");

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&bucket_dir)
            .arg("commit")
            .arg("test commit message 1")
            .assert()
            .success();

        create_test_file(&bucket_dir, "test_file3.txt", "test content 3");

        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&bucket_dir)
            .arg("commit")
            .arg("test commit message 2")
            .assert()
            .success();
        
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(&bucket_dir)
            .arg("history")
            .assert()
            .success()
            .stdout(predicate::str::contains("test commit message 1"))
            .stdout(predicate::str::contains("test commit message 2"));

        
    }


    fn create_test_file(dir: &std::path::Path, filename: &str, content: &str) {
        let file_path = dir.join(filename);
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write to file");
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