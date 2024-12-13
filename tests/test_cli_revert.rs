mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `revert` command.
    ///
    /// # Commands
    /// `$ buckets revert`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_revert() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("revert")
            .assert()
            .success();
    }
}