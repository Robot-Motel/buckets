mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;
    use serial_test::serial;

    /// Test the `link` command.
    ///
    /// # Commands
    /// `$ buckets link test_bucket`
    ///
    /// # Expected output
    ///
    #[test]
    #[serial]
    fn test_cli_link() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("link")
            .assert()
            .success();
    }
}
