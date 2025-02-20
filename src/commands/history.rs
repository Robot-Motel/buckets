use crate::args::HistoryCommand;
use crate::errors::BucketError;
use duckdb::Connection;
use crate::utils::utils::find_bucket_repo;

pub fn execute(command: &HistoryCommand) -> Result<(), BucketError> {
    let x = command;
    println!("History command: {:?}", x);
    let current_dir = std::env::current_dir()?;
    let repo_root = find_bucket_repo(&current_dir).ok_or(BucketError::NotInRepo)?;
    let db_path = repo_root.join("buckets.db");
    
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.message, CAST(c.created_at AS TEXT), b.name as bucket_name 
         FROM commits c 
         JOIN buckets b ON c.bucket_id = b.id 
         ORDER BY c.created_at DESC"
    )?;


    println!("Commit History:");
    println!("----------------------------------------");

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

        println!("Commit ID: {}", id);
        println!("Message: {}", message);
        println!("Created At: {}", created_at);
        println!("Bucket: {}", bucket_name);
        println!("----------------------------------------");
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    use crate::args::HistoryCommand;
    use crate::commands::history::execute;


    #[test]
    fn test_history_command() {
        // Need to setup a proper test environment
        let temp_dir = tempdir().expect("invalid temp dir").into_path();
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

        // change to bucket directory
        env::set_current_dir(&bucket_dir).expect("invalid directory");


        // Test history command
        let history_cmd = HistoryCommand {
            shared: Default::default(),
        };
        let result = execute(&history_cmd);
        
        assert!(result.is_ok());
    }
}


