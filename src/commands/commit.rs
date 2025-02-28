use std::{env, io};
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use blake3::Hash;
use duckdb::params;
use log::{debug, error};
use uuid::Uuid;
use zstd::Encoder;
use crate::args::CommitCommand;
use crate::CURRENT_DIR;
use crate::data::bucket::{Bucket, BucketTrait};
use crate::data::commit::{Commit, CommitStatus, CommittedFile};
use crate::errors::BucketError;
use crate::utils::checks;
use crate::utils::config::RepositoryConfig;
use crate::utils::utils::{connect_to_db, find_bucket_path, find_files_excluding_top_level_b, hash_file};

pub fn execute(commit_command: &CommitCommand) -> Result<(), BucketError> {

    let current_dir = CURRENT_DIR.with(|dir| dir.clone());

    if !checks::is_valid_bucket_repo(&current_dir) {
        return Err(BucketError::NotInRepo);
    }

    let bucket_path = match find_bucket_path(&current_dir) {
        Some(path) => path,
        None => return Err(BucketError::NotAValidBucket),
    };

    let bucket = match Bucket::from_meta_data(&current_dir) {
        Ok(bucket) => bucket,
        Err(e) => {
            error!("Error reading bucket info: {}", e);
            return Err(e);
        }
    };

    let _repo_config = RepositoryConfig::from_file(env::current_dir().expect("invalid directory"))?;

    // create a list of each file in the bucket directory, recursively
    // and create a blake3 hash for each file and add to current_commit
    let current_commit = list_files_with_metadata_in_bucket(bucket_path)?;
    if current_commit.files.is_empty() {
        return Err(Error::new(ErrorKind::NotFound, "No commitable files found in bucket.").into());
    }

    // Load the previous commit, if it exists
    match load_last_commit(bucket.name.clone()) {
        Ok(None) => {
            // There is no previous commit; Process all files in the current commit
            process_files(bucket.id, &bucket.relative_bucket_path, &current_commit.files, &commit_command.message)?;
        }
        Ok(Some(previous_commit)) => {
            // Compare the current commit with the previous commit
            if let Some(changes) = current_commit.compare(&previous_commit) {
                // Process the files that have changed
                process_files(bucket.id, &bucket.get_full_bucket_path()?, &changes, &commit_command.message)?;
            } else {
                // if there are no difference with previous commit cancel commit
                println!("No changes detected. Commit cancelled.");
                return Ok(());
            }
        }
        Err(_) => {
            error!("Failed to load previous commit.");
            return Err(BucketError::from(Error::new(ErrorKind::Other, "Failed to load previous commit.")));
        }
    }

    Ok(())
}

fn process_files(bucket_id: Uuid, bucket_path: &PathBuf, files: &[CommittedFile], message: &String) -> Result<(), BucketError> {
    // Insert the commit into the database
    debug!("bucket id: {}", bucket_id.to_string().to_uppercase());
    let commit_id = insert_commit_into_db(bucket_id, message)?;

    // Create the storage directory
    let storage_path = bucket_path.join(".b").join("storage");

    // Process each file in the commit
    for file in files {
        debug!("Processing file: {} {}", file.name, file.hash);
        let output = storage_path.join(&file.hash.to_string());

        // Insert the file into the database
        insert_file_into_db(&commit_id, &file.name, &file.hash.to_string())?;

        // TODO: Replace unwrap with proper error handling
        // Compress and store the file
        compress_and_store_file(&file.name, output.as_path(), 0)?;
    }
    Ok(())
}

fn insert_file_into_db(commit_id: &str, file_path: &str, hash: &str) -> Result<(), BucketError> {
    let connection = connect_to_db()?;
    let _ = connection.execute(
        "INSERT INTO files (id, commit_id, file_path, hash) VALUES (gen_random_uuid(), ?1, ?2, ?3)",
        [commit_id, file_path, hash],
    )
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Error inserting into database: {}, commit id: {}, file path: {}, hash: {}", e, commit_id, file_path, hash),
            )
        })?;
    Ok(())
}

fn insert_commit_into_db(bucket_id: Uuid, message: &String) -> Result<String, BucketError> {
    let connection = connect_to_db()?;
    debug!("CommitCommand: path to database {}",connection.path().expect("invalid connection path").display());
    // Now query back the `id` using the `rowid`
    let stmt = &mut connection.prepare("INSERT INTO commits (id, bucket_id, message) VALUES (gen_random_uuid(), ?1, ?2) RETURNING id")?;
    let rows = &mut stmt.query(params![bucket_id.to_string().to_uppercase(), message.parse::<String>().unwrap()])?;

    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Err(BucketError::from(duckdb::Error::QueryReturnedNoRows))
    }
}

pub fn compress_and_store_file(input_path: &str, output_path: &Path, compression_level: i32) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    let mut reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);
    let mut encoder = Encoder::new(writer, compression_level)?;

    std::io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?; // Finalize the compression

    Ok(())
}

fn list_files_with_metadata_in_bucket(bucket_path: PathBuf) -> io::Result<Commit> {
    let mut files = Vec::new();

    for entry in find_files_excluding_top_level_b(bucket_path.as_path()) {
        let path = entry.as_path();

        if path.is_file() {
            match hash_file(path) {
                Ok(hash) => {
                    //println!("BLAKE3 hash: {}", hash);
                    files.push(CommittedFile {
                        id: Default::default(),
                        name: path.to_string_lossy().into_owned(),
                        hash,
                        previous_hash: Hash::from_str("0000000000000000000000000000000000000000000000000000000000000000").expect("invalid hash"),
                        status: CommitStatus::Unknown,
                    });
                }
                Err(e) => {
                    eprintln!("Failed to hash file: {}", e);
                    return Err(e);
                }
            }
        } else {
            debug!("Skipping non-file: {:?}", entry.as_path());
        }
    }

    Ok(Commit {
        bucket: "".to_string(),
        files,
        timestamp: chrono::Utc::now().to_rfc3339(),
        previous: None,
        next: None,
    })
}

pub fn load_last_commit(bucket_name: String) -> Result<Option<Commit>, BucketError> {

    let connection = connect_to_db()?;

    let mut stmt = connection.prepare("SELECT f.id, f.file_path, f.hash
                                               FROM files f
                                               JOIN commits c ON f.commit_id = c.id
                                WHERE c.created_at = (SELECT MAX(created_at) FROM commits)")?;

    let mut rows = stmt.query([])?;

    let mut files = Vec::new();
    while let Some(row) = rows.next()? {
        let uuid_string: String = row.get(0)?;
        let hex_string: String = row.get(2)?;

        files.push(CommittedFile {
            id: Uuid::parse_str(&uuid_string).expect("invalid uuid"),
            name: row.get(1)?,
            hash: Hash::from_hex(&hex_string).expect("invalid hash"),
            previous_hash: Hash::from_str("0000000000000000000000000000000000000000000000000000000000000000").expect("invalid hash"), // TODO: Implement previous hash
            status: CommitStatus::Committed,
        });
    }

    Ok(Some(Commit {
        bucket: bucket_name,
        files,
        timestamp: "".to_string(),
        previous: None,
        next: None,
    }))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use blake3::Hash;
    use log::error;
    use tempfile::tempdir;
    use uuid::Uuid;
    use crate::commands::commit::process_files;
    use crate::data::bucket::read_bucket_info;
    use crate::data::commit::{CommitStatus, CommittedFile};

    #[test]
    fn test_process_files() {
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

        // Bucket id is stored in the bucket info file
        // Can be read first to get the bucket id and then use
        // to query the database
        let bucket = read_bucket_info(&bucket_dir).expect("invalid bucket info");

        let commit_message = "Test commit".to_string();
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "test_file.txt".to_string(),
            hash: Hash::from_str("f4315de648c8440fb2539fe9a8417e901ab270a37c6e2267e0c5fffe7d4d4419").expect("invalid hash"),
            previous_hash: Hash::from_str("0000000000000000000000000000000000000000000000000000000000000000").expect("invalid hash"),
            status: CommitStatus::New,
        };

        // change to bucket directory
        env::set_current_dir(&bucket_dir).expect("invalid directory");

        let result = process_files(bucket.id, &bucket_dir, &[committed_file], &commit_message).map_err(
            |e| {
                error!("Error processing files: {}", e);
                e
            }
        );

        assert!(&result.is_ok());

    }

    #[test]
    fn test_compress_and_store_file() {
        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");

        // Create original content and source file
        let original_content = b"This is test content for compression and storage";
        let source_path = temp_dir.path().join("source.txt");
        {
            let mut source_file = File::create(&source_path).expect("Failed to create source file");
            source_file.write_all(original_content).expect("Failed to write to source file");
        }
        
        // Define the compressed output path
        let compressed_path = temp_dir.path().join("1c4fc261196bfcd70efd6d5217a167d86c24cd465f144f15cd41ac336c1106e3");
        
        // Call the function we're testing
        crate::commands::commit::compress_and_store_file(
            source_path.to_str().expect("Invalid path"), 
            &compressed_path, 
            0
        ).expect("Failed to compress file");
        
        // Verify compressed file exists
        assert!(compressed_path.exists(), "Compressed file wasn't created");
        
        // Decompress the file using a proper Decoder
        let compressed_file = File::open(&compressed_path).expect("Failed to open compressed file");
        let mut decoder = zstd::Decoder::new(compressed_file).expect("Failed to create decoder");
        let mut decompressed_content = Vec::new();
        std::io::copy(&mut decoder, &mut decompressed_content).expect("Failed to decompress");
        
        // Compare content
        assert_eq!(decompressed_content, original_content, "Decompressed content doesn't match original");
    }
}