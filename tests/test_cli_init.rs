mod common;
#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `init` command.
    ///
    /// # Commands
    /// `$ buckets init test_repo`
    ///
    /// # Expected output
    /// `.buckets` directory created.
    ///
    /// `.buckets/config` file created.
    ///
    /// `.buckets/buckets.db` database created.
    ///
    #[test]
    fn test_cli_init() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let repo_dir = temp_dir.as_path().join("test_repo");
        assert!(repo_dir.exists());
        assert!(repo_dir.is_dir());

        let repo_dot_buckets_dir = repo_dir.join(".buckets");
        assert!(repo_dot_buckets_dir.exists());
        assert!(repo_dot_buckets_dir.is_dir());

        let repo_config_file = repo_dot_buckets_dir.join("config");
        assert!(repo_config_file.exists());
        assert!(repo_config_file.is_file());

        let repo_database = repo_dot_buckets_dir.join("buckets.db");
        assert!(repo_database.exists());
        assert!(repo_database.is_file());
    }
}