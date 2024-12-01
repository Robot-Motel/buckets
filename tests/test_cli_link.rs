mod common;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the `link` command.
    ///
    /// # Commands
    /// `$ buckets link test_bucket`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_link() {
        let temp_dir = common::get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.current_dir(temp_dir.as_path())
            .arg("link")
            .assert()
            .success();
    }
}
