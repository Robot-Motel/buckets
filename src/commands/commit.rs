use crate::args::CommitCommand;
use crate::commands::BucketCommand;
use crate::data::bucket::BucketTrait;
use crate::data::commit::{Commit as CommitData, CommitStatus, CommittedFile};
use crate::errors::BucketError;
use crate::utils::compression::compress_and_store_file;
use crate::utils::utils::{connect_to_db, find_files_excluding_top_level_b, hash_file};
use crate::world::World;
use blake3::Hash;
use duckdb::params;
use log::{debug, error};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

/// Commit changes to a bucket
pub struct Commit {
    args: CommitCommand,
}

impl BucketCommand for Commit {
    type Args = CommitCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        let world = World::new(&self.args.shared)?;

        let bucket = match &world.bucket {
            Some(bucket) => bucket,
            None => return Err(BucketError::NotInBucket),
        };

        // create a list of each file in the bucket directory, recursively
        // and create a blake3 hash for each file and add to current_commit
        let current_commit =
            self.list_files_with_metadata_in_bucket(bucket.get_full_bucket_path()?)?;
        if current_commit.files.is_empty() {
            return Err(
                Error::new(ErrorKind::NotFound, "No commitable files found in bucket.").into(),
            );
        }

        // Load the previous commit, if it exists
        match Commit::load_last_commit(bucket.name.clone()) {
            Ok(None) => {
                // There is no previous commit; Process all files in the current commit
                self.process_files(
                    bucket.id,
                    &bucket.relative_bucket_path,
                    &current_commit.files,
                    &self.args.message,
                )?;
            }
            Ok(Some(previous_commit)) => {
                // Compare the current commit with the previous commit
                if let Some(changes) = current_commit.compare(&previous_commit) {
                    // Process the files that have changed
                    self.process_files(
                        bucket.id,
                        &bucket.get_full_bucket_path()?,
                        &changes,
                        &self.args.message,
                    )?;
                } else {
                    // if there are no difference with previous commit cancel commit
                    println!("No changes detected. Commit cancelled.");
                    return Ok(());
                }
            }
            Err(_) => {
                error!("Failed to load previous commit.");
                return Err(BucketError::from(Error::new(
                    ErrorKind::Other,
                    "Failed to load previous commit.",
                )));
            }
        }

        Ok(())
    }
}

impl Commit {
    pub fn process_files(
        &self,
        bucket_id: Uuid,
        bucket_path: &PathBuf,
        files: &[CommittedFile],
        message: &String,
    ) -> Result<(), BucketError> {
        // Insert the commit into the database
        debug!("bucket id: {}", bucket_id.to_string().to_uppercase());
        let commit_id = self.insert_commit_into_db(bucket_id, message)?;

        // Create the storage directory
        let storage_path = bucket_path.join(".b").join("storage");

        // Process each file in the commit
        for file in files {
            debug!("Processing file: {} {}", file.name, file.hash);
            let output = storage_path.join(&file.hash.to_string());

            // Insert the file into the database
            self.insert_file_into_db(&commit_id, &file.name, &file.hash.to_string())?;

            // Compress and store the file
            compress_and_store_file(&file.name, output.as_path(), 0)
                .map_err(|e| {
                    error!("Error compressing and storing file: {}", e);
                    e
                })?;
        }
        Ok(())
    }

    fn insert_file_into_db(
        &self,
        commit_id: &str,
        file_path: &str,
        hash: &str,
    ) -> Result<(), BucketError> {
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
        connection.close().expect("failed to close connection");
        Ok(())
    }

    fn insert_commit_into_db(
        &self,
        bucket_id: Uuid,
        message: &String,
    ) -> Result<String, BucketError> {
        let connection = connect_to_db()?;
        debug!(
            "CommitCommand: path to database {}",
            connection
                .path()
                .expect("invalid connection path")
                .display()
        );
        // Now query back the `id` using the `rowid`
        let stmt = &mut connection.prepare("INSERT INTO commits (id, bucket_id, message) VALUES (gen_random_uuid(), ?1, ?2) RETURNING id")?;
        let rows = &mut stmt.query(params![
            bucket_id.to_string().to_uppercase(),
            message.parse::<String>().unwrap()
        ])?;

        let result = if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Err(BucketError::from(duckdb::Error::QueryReturnedNoRows))
        };
        
        connection.close().expect("failed to close connection");
        result
    }



    fn list_files_with_metadata_in_bucket(&self, bucket_path: PathBuf) -> io::Result<CommitData> {
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
                            previous_hash: Hash::from_str(
                                "0000000000000000000000000000000000000000000000000000000000000000",
                            )
                            .expect("invalid hash"),
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

        Ok(CommitData {
            bucket: "".to_string(),
            files,
            timestamp: chrono::Utc::now().to_rfc3339(),
            previous: None,
            next: None,
        })
    }

    pub fn load_last_commit(bucket_name: String) -> Result<Option<CommitData>, BucketError> {
        let connection = connect_to_db()?;

        let mut stmt = connection.prepare(
            "SELECT f.id, f.file_path, f.hash
                                               FROM files f
                                               JOIN commits c ON f.commit_id = c.id
                                WHERE c.created_at = (SELECT MAX(created_at) FROM commits)",
        )?;

        let mut rows = stmt.query([])?;

        let mut files = Vec::new();
        while let Some(row) = rows.next()? {
            let uuid_string: String = row.get(0)?;
            let hex_string: String = row.get(2)?;

            files.push(CommittedFile {
                id: Uuid::parse_str(&uuid_string).expect("invalid uuid"),
                name: row.get(1)?,
                hash: Hash::from_hex(&hex_string).expect("invalid hash"),
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .expect("invalid hash"), // TODO: Implement previous hash
                status: CommitStatus::Committed,
            });
        }

        connection.close().expect("failed to close connection");

        Ok(Some(CommitData {
            bucket: bucket_name,
            files,
            timestamp: "".to_string(),
            previous: None,
            next: None,
        }))
        
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::commit::Commit;
    use crate::commands::BucketCommand;
    use crate::data::bucket::read_bucket_info;
    use crate::data::commit::{CommitStatus, CommittedFile};
    use blake3::Hash;
    use log::error;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::tempdir;
    use uuid::Uuid;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_process_files() {
        // Need to setup a proper test environment
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

        // Bucket id is stored in the bucket info file
        // Can be read first to get the bucket id and then use
        // to query the database
        let bucket = read_bucket_info(&bucket_dir).expect("invalid bucket info");

        let commit_message = "Test commit".to_string();
        let committed_file = CommittedFile {
            id: Uuid::new_v4(),
            name: "test_file.txt".to_string(),
            hash: Hash::from_str(
                "f4315de648c8440fb2539fe9a8417e901ab270a37c6e2267e0c5fffe7d4d4419",
            )
            .expect("invalid hash"),
            previous_hash: Hash::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("invalid hash"),
            status: CommitStatus::New,
        };

        // change to bucket directory
        env::set_current_dir(&bucket_dir).expect("invalid directory");

        let commit_cmd = Commit::new(&crate::args::CommitCommand {
            shared: crate::args::SharedArguments::default(),
            message: commit_message.clone(),
        });
        let result = commit_cmd
            .process_files(bucket.id, &bucket_dir, &[committed_file], &commit_message)
            .map_err(|e| {
                error!("Error processing files: {}", e);
                e
            });

        match result {
            Ok(_) => (),
            Err(e) => {
                panic!("Error processing files: {}", e);
            }
        }
    }
}
