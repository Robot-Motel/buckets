use crate::data::commit::{Commit, CommitStatus, CommittedFile};
use crate::errors::BucketError;
use crate::utils::checks::{find_directory_in_parents, is_valid_bucket_info};
use crate::utils::utils::{
    connect_to_db, find_bucket_repo, find_files_excluding_top_level_b, hash_file,
};
use blake3::Hash;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, io};
use toml::to_string;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Bucket {
    pub id: Uuid,
    pub name: String,
    pub relative_bucket_path: PathBuf,
}

pub trait BucketTrait {
    fn default(uuid: Uuid, name: &String, path: &PathBuf) -> Self;
    fn from_meta_data(current_path: &PathBuf) -> Result<Bucket, BucketError>;
    fn write_bucket_info(&self) -> Result<(), io::Error>;
    fn is_valid_bucket(dir_path: &Path) -> bool;
    fn find_bucket(dir_path: &Path) -> Option<PathBuf>;
    fn get_full_bucket_path(&self) -> Result<PathBuf, BucketError>;
    #[allow(dead_code)]
    fn list_files_with_metadata_in_bucket(&self) -> io::Result<Commit>;
    #[allow(dead_code)]
    fn load_last_commit(&self) -> Result<Option<Commit>, BucketError>;
}

impl BucketTrait for Bucket {
    fn default(uuid: Uuid, name: &String, path: &PathBuf) -> Bucket {
        Bucket {
            id: uuid,
            name: name.to_string(),
            relative_bucket_path: path.to_path_buf(),
        }
    }

    fn from_meta_data(current_path: &PathBuf) -> Result<Self, BucketError> {
        debug!("Current path {}", current_path.as_path().display());
        // find the top level of the bucket directory
        let bucket_path: PathBuf = match Bucket::find_bucket(current_path.as_path()) {
            Some(mut path) => {
                path.pop();
                path
            }
            None => {
                return Err(BucketError::NotAValidBucket);
            }
        };

        let bucket = read_bucket_info(&bucket_path)?;

        // check if it is a valid bucket
        if !Self::is_valid_bucket(bucket_path.as_path()) {
            return Err(BucketError::NotAValidBucket);
        }

        Ok(bucket)
    }

    fn write_bucket_info(&self) -> Result<(), io::Error> {
        let mut file = File::create(self.relative_bucket_path.join(".b").join("info"))?;
        let serialized = to_string(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        file.write_fmt(format_args!("{}", serialized))?;
        Ok(())
    }

    fn is_valid_bucket(dir_path: &Path) -> bool {
        let bucket_path = find_directory_in_parents(dir_path, ".b");
        match bucket_path {
            Some(path) => is_valid_bucket_info(&path),
            None => false,
        }
    }

    fn find_bucket(dir_path: &Path) -> Option<PathBuf> {
        match find_directory_in_parents(dir_path, ".b") {
            Some(path) => Some(path),
            None => None,
        }
    }

    fn get_full_bucket_path(&self) -> Result<PathBuf, BucketError> {
        let current_dir = env::current_dir().map_err(BucketError::from)?;
        let full_bucket_path = find_bucket_repo(&current_dir.as_path())
            .ok_or(BucketError::NotInRepo)?
            .parent()
            .ok_or(BucketError::NotInRepo)?
            .join(&self.relative_bucket_path);
        Ok(full_bucket_path)
    }

    fn list_files_with_metadata_in_bucket(&self) -> io::Result<Commit> {
        let mut files = Vec::new();

        for entry in find_files_excluding_top_level_b(
            self.get_full_bucket_path()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .as_path(),
        ) {
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
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
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

    fn load_last_commit(&self) -> Result<Option<Commit>, BucketError> {
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
                id: Uuid::parse_str(&uuid_string)
                    .map_err(|e| BucketError::InvalidData(e.to_string()))?,
                name: row.get(1)?,
                hash: Hash::from_hex(&hex_string)
                    .map_err(|e| BucketError::InvalidData(e.to_string()))?,
                previous_hash: Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .map_err(|e| BucketError::InvalidData(e.to_string()))?,
                status: CommitStatus::Committed,
            });
        }

        if let Err((_conn, e)) = connection.close() {
            return Err(BucketError::from(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to close database connection: {}", e),
            )));
        }

        Ok(Some(Commit {
            bucket: self.name.clone(),
            files,
            timestamp: "".to_string(),
            previous: None,
            next: None,
        }))
    }
}

pub fn read_bucket_info(path: &PathBuf) -> Result<Bucket, std::io::Error> {
    let info_path = path.join(".b").join("info");
    let mut file = File::open(&info_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to open {} file: {}",
                &info_path.as_os_str().to_str().unwrap_or("<invalid path>"),
                e
            ),
        )
    })?;
    let mut toml_string = String::new();
    file.read_to_string(&mut toml_string)?;

    let bucket = toml::from_str(&toml_string)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(bucket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs::create_dir_all;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use uuid::Uuid;

    // Helper function to set up test environment
    fn setup_test_environment() -> std::io::Result<PathBuf> {
        let temp_dir = tempdir()?.keep();
        create_dir_all(temp_dir.as_path().join(".buckets"))?;
        let bucket_path = temp_dir.as_path().join("test_bucket");
        create_dir_all(&bucket_path)?;
        env::set_current_dir(&bucket_path)?;
        Ok(bucket_path)
    }

    #[test]
    fn test_default() {
        let name = String::from("test_bucket");
        let path = PathBuf::from("/some/path/.b");

        let bucket = Bucket::default(Uuid::new_v4(), &name, &path);

        assert_eq!(bucket.name, name);
        assert_eq!(bucket.relative_bucket_path, path);
    }

    #[test]
    fn test_write_and_read_bucket_info() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_name = String::from("test_bucket");
        let bucket_path = temp_dir.path().to_path_buf().join(&bucket_name);
        let bucket_meta_path = bucket_path.join(".b");
        create_dir_all(&bucket_meta_path)?;

        let bucket_default = Bucket::default(Uuid::new_v4(), &bucket_name, &bucket_path);
        bucket_default
            .write_bucket_info()
            .expect("Failed to write bucket info in test");

        let bucket = match Bucket::from_meta_data(&bucket_path) {
            Ok(bucket) => bucket,
            Err(e) => panic!("Error reading bucket info: {}", e),
        };

        assert_eq!(bucket_default, bucket);
        Ok(())
    }

    #[test]
    fn test_bucket_serialization_roundtrip() {
        let bucket = Bucket {
            id: Uuid::new_v4(),
            name: "test_bucket".to_string(),
            relative_bucket_path: PathBuf::from("path/to/bucket"),
        };

        let toml_string = toml::to_string(&bucket).expect("Failed to serialize bucket");
        let deserialized: Bucket =
            toml::from_str(&toml_string).expect("Failed to deserialize bucket");

        assert_eq!(bucket.id, deserialized.id);
        assert_eq!(bucket.name, deserialized.name);
        assert_eq!(
            bucket.relative_bucket_path,
            deserialized.relative_bucket_path
        );
    }

    #[test]
    fn test_bucket_from_meta_data_invalid_path() {
        let nonexistent_path = PathBuf::from("/definitely/does/not/exist");
        let result = Bucket::from_meta_data(&nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_bucket_from_meta_data_no_info_file() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().join("bucket_no_info");
        let bucket_meta_path = bucket_path.join(".b");
        create_dir_all(&bucket_meta_path)?;

        let result = Bucket::from_meta_data(&bucket_path);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_bucket_from_meta_data_corrupted_info() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().join("bucket_corrupted");
        let bucket_meta_path = bucket_path.join(".b");
        create_dir_all(&bucket_meta_path)?;

        // Write invalid TOML to info file
        let info_path = bucket_meta_path.join("info");
        std::fs::write(&info_path, "invalid toml content { [ ] }")?;

        let result = Bucket::from_meta_data(&bucket_path);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_bucket_write_info_permission_denied() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().join("bucket_readonly");
        let bucket_meta_path = bucket_path.join(".b");
        create_dir_all(&bucket_meta_path)?;

        let bucket = Bucket::default(Uuid::new_v4(), &"test".to_string(), &bucket_path);

        // Make directory read-only on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&bucket_meta_path)?.permissions();
            perms.set_mode(0o444); // read-only
            std::fs::set_permissions(&bucket_meta_path, perms)?;

            let result = bucket.write_bucket_info();

            // Restore permissions for cleanup
            let mut perms = std::fs::metadata(&bucket_meta_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&bucket_meta_path, perms)?;

            assert!(result.is_err());
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_bucket_get_full_bucket_path() -> std::io::Result<()> {
        let bucket_path = setup_test_environment()?;
        let bucket = Bucket::default(Uuid::new_v4(), &"test".to_string(), &bucket_path);
        // change the current directory to the bucket path

        match bucket.get_full_bucket_path() {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to get full bucket path: {:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, e))
            }
        }
    }

    #[test]
    #[serial]
    fn test_bucket_list_files_with_metadata_in_bucket_empty() -> std::io::Result<()> {
        let bucket_path = setup_test_environment()?;

        let bucket = Bucket::default(Uuid::new_v4(), &"empty".to_string(), &bucket_path);
        match bucket.list_files_with_metadata_in_bucket() {
            Ok(files) => {
                assert!(files.files.is_empty());
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to list files in bucket: {:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, e))
            }
        }
    }

    #[test]
    #[serial]
    fn test_bucket_list_files_with_metadata_in_bucket_with_files() -> std::io::Result<()> {
        let bucket_path = setup_test_environment()?;

        // Create some test files
        std::fs::write(bucket_path.join("file1.txt"), "content1")?;
        std::fs::write(bucket_path.join("file2.txt"), "content2")?;

        let bucket = Bucket::default(Uuid::new_v4(), &"with_files".to_string(), &bucket_path);
        match bucket.list_files_with_metadata_in_bucket() {
            Ok(files) => {
                assert_eq!(files.files.len(), 2);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to list files in bucket: {:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, e))
            }
        }
    }

    #[test]
    #[serial]
    fn test_bucket_list_files_with_metadata_in_bucket_with_subdirectories() -> std::io::Result<()> {
        let bucket_path = setup_test_environment()?;

        // Create test files in subdirectories
        let subdir = bucket_path.join("subdir");
        create_dir_all(&subdir)?;
        std::fs::write(subdir.join("file1.txt"), "content1")?;
        std::fs::write(subdir.join("file2.txt"), "content2")?;

        let bucket = Bucket::default(Uuid::new_v4(), &"with_subdirs".to_string(), &bucket_path);
        match bucket.list_files_with_metadata_in_bucket() {
            Ok(files) => {
                assert_eq!(files.files.len(), 2);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to list files in bucket: {:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, e))
            }
        }
    }

    #[test]
    fn test_read_bucket_info_invalid_path() {
        let nonexistent_path = PathBuf::from("/definitely/does/not/exist");
        let result = read_bucket_info(&nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_bucket_info_valid_file() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().join("valid_bucket");
        let bucket_meta_path = bucket_path.join(".b");
        create_dir_all(&bucket_meta_path)?;

        // Create a valid bucket and write its info
        let original_bucket =
            Bucket::default(Uuid::new_v4(), &"valid_bucket".to_string(), &bucket_path);
        original_bucket.write_bucket_info()?;

        // Read it back using the standalone function
        let read_bucket = read_bucket_info(&bucket_path)?;
        assert_eq!(original_bucket, read_bucket);
        Ok(())
    }

    #[test]
    fn test_bucket_fields() {
        let uuid = Uuid::new_v4();
        let name = "test_bucket".to_string();
        let path = PathBuf::from("test/path");

        let bucket = Bucket {
            id: uuid,
            name: name.clone(),
            relative_bucket_path: path.clone(),
        };

        assert_eq!(bucket.id, uuid);
        assert_eq!(bucket.name, name);
        assert_eq!(bucket.relative_bucket_path, path);
    }
}
