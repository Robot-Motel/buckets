mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `finalize` command.
    ///
    /// # Commands
    /// `$ buckets finalize`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_finalize() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("finalize")
            .assert()
            .success();
    }
}