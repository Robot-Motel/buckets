mod common;
#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    /// Test the `schema` command.
    ///
    /// # Commands
    /// `$ buckets schema`
    ///
    /// # Expected output
    /// Prints the SQL schema used to create the database
    ///
    #[test]
    fn test_cli_schema() {
        let mut cmd = Command::cargo_bin("buckets").expect("failed to run command");
        
        // Execute the schema command and verify it succeeds
        let assert = cmd.arg("schema").assert().success();
        
        // Verify the output contains expected SQL keywords
        assert
            .stdout(predicate::str::contains("CREATE TABLE buckets"))
            .stdout(predicate::str::contains("CREATE TABLE commits"))
            .stdout(predicate::str::contains("CREATE TABLE files"));
    }
} 