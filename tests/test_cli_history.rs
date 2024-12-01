mod common;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the `history` command.
    ///
    /// # Commands
    /// `$ buckets history`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_history() {
        let temp_dir = common::get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("history")
            .assert()
            .success();
    }
}