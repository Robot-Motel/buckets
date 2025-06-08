use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use log::{debug, error};
use crate::args::RestoreCommand;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::errors::BucketError;
use crate::utils::checks;
use crate::utils::utils::{connect_to_db, find_bucket_path};
use crate::commands::BucketCommand;

/// Restore a file from the last commit
pub struct Restore {
    args: RestoreCommand,
}

impl BucketCommand for Restore {
    type Args = RestoreCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
    let current_dir = CURRENT_DIR.with(|dir| dir.clone());

    if !checks::is_valid_bucket_repo(&current_dir) {
        return Err(BucketError::NotInRepo);
    }

    let bucket_path = match find_bucket_path(&current_dir) {
        Some(path) => path,
        None => return Err(BucketError::NotAValidBucket),
    };

    let bucket = Bucket::from_meta_data(&current_dir)?;

    // Get the file's hash from the last commit
    let connection = connect_to_db()?;
    let mut stmt = connection.prepare(
        "SELECT f.hash 
        FROM files f
        JOIN commits c ON f.commit_id = c.id
        WHERE f.file_path = ?1
        AND c.created_at = (
            SELECT MAX(created_at) 
            FROM commits 
            WHERE bucket_id = ?2
        )"
    )?;
        let file_path = self.args.file.clone();
    let relative_path = PathBuf::from(&file_path)
        .strip_prefix(&bucket_path)
        .unwrap_or(&PathBuf::from(&file_path))
        .to_string_lossy()
        .to_string();
    let rows = stmt.query_map([&relative_path, &bucket.id.to_string()], |row| {
        row.get::<_, String>(0)
    })?;

    let hash = match rows.last() {
        Some(Ok(hash)) => hash,
        _ => return Err(BucketError::FileNotFound(file_path)),
    };

    // Construct paths
    let storage_path = bucket_path.join(".b").join("storage").join(&hash);
    let target_path = PathBuf::from(&file_path);

    debug!("Restoring {} from {}", target_path.display(), storage_path.display());

    // Decompress and copy the file from storage
        self.decompress_and_restore_file(&storage_path, &target_path)
        .map_err(|e| {
            error!("Failed to restore file: {}", e);
            BucketError::from(e)
        })?;

    println!("Restored {}", file_path);
    connection.close().expect("failed to close connection");
    Ok(())
    }
}

impl Restore {
    fn decompress_and_restore_file(&self, storage_path: &PathBuf, target_path: &PathBuf) -> std::io::Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Open the compressed file
    let input_file = File::open(storage_path)?;
    let reader = BufReader::new(input_file);
    
    // Delete the target file if it exists
    if target_path.exists() {
        std::fs::remove_file(target_path)?;
    }

    // Create the output file
    let output_file = File::create(target_path)?;
    let writer = BufWriter::new(output_file);
    
    // Create a zstd decoder
    let mut decoder = zstd::Decoder::new(reader)?;

    // Copy data from decoder to output 
    std::io::copy(&mut decoder, &mut BufWriter::new(writer))?;
    Ok(())
    }
}

// Keep the old function for backward compatibility during transition
pub fn execute(command: RestoreCommand) -> Result<(), BucketError> {
    let cmd = Restore::new(&command);
    cmd.execute()
}

#[cfg(test)]
mod tests {
    use crate::commands::commit::Commit;
    use serial_test::serial;

    use super::*;
    use std::{env, fs};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn test_restore_command() {
        // Setup test environment
        let temp_dir = tempdir().expect("invalid temp dir").into_path();
        log::debug!("temp_dir: {:?}", temp_dir);
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
        let original_content = b"original content";
        file.write_all(original_content).expect("invalid write");


        let mut cmd3 = assert_cmd::Command::cargo_bin("buckets").expect("invalid command");
        cmd3.current_dir(bucket_dir.as_path())
            .arg("commit")
            .arg("test message")
            .assert()
            .success();

        // Modify the file
        let modified_content = b"modified content";
        let mut file = File::create(&file_path).unwrap();
        file.write_all(modified_content).unwrap();

        // Change to bucket directory
        env::set_current_dir(&bucket_dir).expect("invalid directory");

        // Restore the file
        let restore_cmd = RestoreCommand {
            file: file_path.to_str().unwrap().to_string(),
            shared: Default::default(),
        };
        execute(restore_cmd).unwrap();

        // Verify the file was restored using binary comparison
        let restored_content = fs::read(&file_path).expect("invalid read");
        assert_eq!(restored_content, original_content);
    }

    #[test]
    #[serial]
    fn test_decompress_and_restore_file() {
        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");
        
        // Create original content
        let original_content = b"original content";
        
        // Create source file path
        let source_path = temp_dir.path().join("source.txt");
        
        // Write original content to source file
        std::fs::write(&source_path, original_content).expect("Failed to write source file");
        
        // Create compressed file path
        let compressed_path = temp_dir.path().join("compressed.zst");

        // Compress and store the file
        let commit_cmd = Commit::new(&crate::args::CommitCommand {
            shared: crate::args::SharedArguments::default(),
            message: "test".to_string(),
        });
        commit_cmd.compress_and_store_file(&source_path.to_str().unwrap(), &compressed_path, 0).expect("Failed to compress and store file");
        
        // Create restored file path
        let restored_path = temp_dir.path().join("restored.txt");
        
        // Call the function we're testing
        let restore_cmd = Restore::new(&RestoreCommand {
            shared: crate::args::SharedArguments::default(),
            file: "test".to_string(),
        });
        restore_cmd.decompress_and_restore_file(
            &compressed_path, 
            &restored_path
        ).expect("Failed to decompress and restore file");
        
        // Read the restored content
        let restored_content = std::fs::read(&restored_path).expect("Failed to read restored file");
        
        // Compare content
        assert_eq!(restored_content, original_content, "Restored content doesn't match original");
    }
}