mod common;

#[cfg(test)]
mod tests {
    /// Test the `stats` command.
    ///
    /// # Commands
    /// `$ typst stats`
    ///
    /// # Expected output
    ///
    #[test]
    fn test_cli_stats() {
        let mut cmd = assert_cmd::Command::cargo_bin("buckets").unwrap();
        cmd.arg("stats")
            .assert()
            .success();
    }
}