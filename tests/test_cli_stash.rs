mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;
    use serial_test::serial;

    /// Test the `stash` command.
    ///
    /// # Commands
    /// `$ buckets stash`
    ///
    /// # Expected output
    ///
    #[test]
    #[serial]
    fn test_cli_stash() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("stash")
            .assert()
            .success();
    }
}
