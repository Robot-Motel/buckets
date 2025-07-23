mod common;
#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;
    use serial_test::serial;

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
    /// `.buckets/postgres_data` directory created.
    ///
    #[test]
    #[serial]
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

        let repo_database_dir = repo_dot_buckets_dir.join("postgres_data");
        assert!(repo_database_dir.exists());
        assert!(repo_database_dir.is_dir());
    }

    /// Test the `init` command with external database.
    ///
    /// # Commands
    /// `$ buckets init test_repo --external-database "postgres://user:password@localhost/db"`
    ///
    /// # Expected output
    /// `.buckets` directory created.
    ///
    /// `.buckets/config` file created with external database connection string.
    ///
    #[test]
    #[serial]
    fn test_cli_init_external_database() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .arg("--external-database")
            .arg("postgres://user:password@localhost/db")
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

        let config_content = std::fs::read_to_string(repo_config_file).expect("failed to read config file");
        assert!(config_content.contains("external_database = \"postgres://user:password@localhost/db\""));
    }
}
