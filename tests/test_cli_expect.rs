mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `expect` command.
    ///
    /// # Commands
    /// `$ typst expect`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_expect() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("expect")
            .assert()
            .success();
    }
}