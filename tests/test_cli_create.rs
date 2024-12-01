mod common;
#[cfg(test)]
mod tests {
    use super::*;

    /// Test the `create` command.
    ///
    /// # Commands
    /// `$ buckets create test_repo`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_init() {
        let temp_dir = common::get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success();
    }
}