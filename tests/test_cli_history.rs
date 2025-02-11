mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `history` command.
    ///
    /// # Commands
    /// `$ buckets history`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_history() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("history")
            .assert()
            .success();
    }
}