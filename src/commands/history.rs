use std::path::PathBuf;

use crate::args::HistoryCommand;
use crate::errors::BucketError;
use duckdb::Connection;
use crate::utils::utils::find_bucket_repo;

#[derive(Debug)]
pub struct CommitRecord {
    id: String,
    message: String,
    created_at: String,
    bucket_name: String,
}

impl CommitRecord {
    pub fn new(id: String, message: String, created_at: String, bucket_name: String) -> Self {
        Self {
            id,
            message,
            created_at,
            bucket_name,
        }
    }

    pub fn display(&self) {
        println!("Commit ID: {}", self.id);
        println!("Message: {}", self.message);
        println!("Created At: {}", self.created_at);
        println!("Bucket: {}", self.bucket_name);
        println!("----------------------------------------");
    }
}

pub fn execute(command: HistoryCommand) -> Result<(), BucketError> {
    let x = command;
    println!("History command: {:?}", x);
    
    let current_dir = std::env::current_dir()?;
    let commits = fetch_commit_history(&current_dir)?;
    display_commit_history(&commits);
    
    Ok(())
}

fn fetch_commit_history(bucket_dir: &PathBuf) -> Result<Vec<CommitRecord>, BucketError> {
    
    let repo_root = find_bucket_repo(&bucket_dir).ok_or(BucketError::NotInRepo)?;
    let db_path = repo_root.join("buckets.db");
    
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.message, CAST(c.created_at AS TEXT), b.name as bucket_name 
         FROM commits c 
         JOIN buckets b ON c.bucket_id = b.id 
         ORDER BY c.created_at DESC"
    )?;

    let mut commits = Vec::new();
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let message: String = row.get(1)?;
        let created_at: String = match row.get(2) {
            Ok(it) => it,
            Err(err) => return Err(BucketError::InvalidData(format!("Invalid data: {:?}", err.to_string()))),
        };
        let bucket_name: String = match row.get(3) {
            Ok(it) => it,
            Err(err) => return Err(BucketError::InvalidData(format!("Invalid data: {:?}", err.to_string()))),
        };

        commits.push(CommitRecord::new(id, message, created_at, bucket_name));
    }

    Ok(commits)
}

fn display_commit_history(commits: &[CommitRecord]) {
    println!("Commit History:");
    println!("----------------------------------------");
    
    for commit in commits {
        commit.display();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use crate::args::HistoryCommand;
    use serial_test::serial;
    
    #[test]
    fn test_commit_record_display() {
        let record = CommitRecord::new(
            "abc123".to_string(),
            "Test commit".to_string(),
            "2023-01-01 12:00:00".to_string(),
            "test_bucket".to_string()
        );
        
        // This is a simple test that just ensures the display method doesn't panic
        record.display();
    }
    
    #[test]
    #[serial]
    fn test_fetch_commit_history() {
        // Setup test environment
        let temp_dir = tempdir().expect("invalid temp dir").keep();
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let mut cmd2 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        let repo_dir = temp_dir.as_path().join("test_repo");
        cmd2.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success();

        let bucket_dir = repo_dir.join("test_bucket");
        let file_path = bucket_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("invalid file");
        file.write_all(b"test").expect("invalid write");
        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        // Test fetch_commit_history
        let commits = fetch_commit_history(&bucket_dir).expect("Failed to fetch commit history");
        
        // Verify we have at least one commit
        assert!(!commits.is_empty());
        
        // Verify the commit has the expected message
        assert!(commits.iter().any(|c| c.message == "test message"));
    }

    #[test]
    #[serial]
    fn test_history_command() {
        // Setup test environment
        let temp_dir = tempdir().expect("invalid temp dir").keep();
        let mut cmd1 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        cmd1.current_dir(temp_dir.as_path())
            .arg("init")
            .arg("test_repo")
            .assert()
            .success();

        let mut cmd2 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        let repo_dir = temp_dir.as_path().join("test_repo");
        cmd2.current_dir(repo_dir.as_path())
            .arg("create")
            .arg("test_bucket")
            .assert()
            .success();

        let bucket_dir = repo_dir.join("test_bucket");
        let file_path = bucket_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("invalid file");
        file.write_all(b"test").expect("invalid write");
        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        // Change to bucket directory
        env::set_current_dir(&bucket_dir).expect("invalid directory");

        // Test history command
        let history_cmd = HistoryCommand {
            shared: Default::default(),
        };
        let result = execute(history_cmd);
        
        assert!(result.is_ok());
    }
}


