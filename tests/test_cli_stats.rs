mod common;

#[cfg(test)]
mod tests {
    use serial_test::serial;

    /// Test the `stats` command.
    ///
    /// # Commands
    /// `$ typst stats`
    ///
    /// # Expected output
    ///
    #[test]
    #[serial]
    fn test_cli_stats() {
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").expect("failed to run command");
        cmd.arg("stats").assert().success();
    }
}
