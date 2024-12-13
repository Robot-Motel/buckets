mod common;

#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;

    /// Test the `rollback` command.
    ///
    /// # Commands
    /// `$ buckets rollback`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_rollback() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        // cmd.current_dir(temp_dir.as_path())
        //     .arg("rollback")
        //     .assert()
        //     .success();
    }
}