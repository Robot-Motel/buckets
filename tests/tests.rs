mod common;

/// Test the `check` command.
///
/// # Commands
/// `$ typst check`
///
/// # Expected output
///
#[cfg(test)]
mod tests {
    use crate::common::tests::get_test_dir;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_version() {
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.arg("--version").assert().success();
    }

    #[test]
    #[serial]
    fn test_cli_check() {
        let temp_dir = get_test_dir();
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.current_dir(temp_dir.as_path())
            .arg("check")
            .assert()
            .success();
    }
}
