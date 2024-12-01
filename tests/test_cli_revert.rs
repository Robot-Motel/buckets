mod common;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the `revert` command.
    ///
    /// # Commands
    /// `$ buckets revert`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_revert() {
        let temp_dir = common::get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("revert")
            .assert()
            .success();
    }
}