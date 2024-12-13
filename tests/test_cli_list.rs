mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `list` command.
    ///
    /// # Commands
    /// `$ buckets list`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_list() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("list")
            .assert()
            .success();
    }
}