use crate::utils::checks::{find_directory_in_parents, is_valid_bucket_info};
use log::debug;
use std::fs::File;
use std::{env, io};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use blake3::Hash;
use serde::{Deserialize, Serialize};
use toml::to_string;
use uuid::Uuid;
use crate::data::commit::{Commit, CommitStatus, CommittedFile};
use crate::errors::BucketError;
use crate::utils::utils::{connect_to_db, find_bucket_repo, find_files_excluding_top_level_b, hash_file};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Bucket {
    pub id: Uuid,
    pub name: String,
    pub relative_bucket_path: PathBuf,
}

pub trait BucketTrait {
    fn default(uuid: Uuid, name: &String, path: &PathBuf) -> Self;
    fn from_meta_data(current_path: &PathBuf) -> Result<Bucket, BucketError>;
    fn write_bucket_info(&self);
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

    fn write_bucket_info(&self) {
        let mut file = File::create(self.relative_bucket_path.join(".b").join("info")).expect("Failed to create bucket info file");
        file.write_fmt(format_args!("{}", to_string(self).expect("Failed to serialize bucket info"))).expect("Failed to write bucket info");
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

        for entry in find_files_excluding_top_level_b(self.get_full_bucket_path()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?.
            as_path()) {
            let path = entry.as_path();

            if path.is_file() {
                match hash_file(path) {
                    Ok(hash) => {
                        //println!("BLAKE3 hash: {}", hash);
                        files.push(CommittedFile {
                            id: Default::default(),
                            name: path.to_string_lossy().into_owned(),
                            hash,
                            previous_hash: Hash::from_str("0000000000000000000000000000000000000000000000000000000000000000")
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
                id: Uuid::parse_str(&uuid_string).map_err(|e| BucketError::InvalidData(e.to_string()))?,
                name: row.get(1)?,
                hash: Hash::from_hex(&hex_string).map_err(|e| BucketError::InvalidData(e.to_string()))?,
                previous_hash: Hash::from_str("0000000000000000000000000000000000000000000000000000000000000000").expect("Failed to create hash"),
                status: CommitStatus::Committed,
            });
        }

        connection.close().expect("failed to close connection");

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
                &info_path.as_os_str().to_str().expect("Failed to convert path to string"),
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
    use std::fs::create_dir_all;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use uuid::Uuid;

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
        bucket_default.write_bucket_info();

        let bucket = match Bucket::from_meta_data(&bucket_path) {
            Ok(bucket) => bucket,
            Err(e) => panic!("Error reading bucket info: {}", e),
        };

        assert_eq!(bucket_default, bucket);
        Ok(())
    }
}
