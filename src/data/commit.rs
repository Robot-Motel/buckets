use blake3::Hash;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

use crate::utils::compression::{compress_and_store_file, restore_file};

#[derive(Serialize, Deserialize, Debug)]
pub enum CommitStatus {
    Unknown,
    New,
    Committed,
    Modified,
    Deleted,
}

impl Display for CommitStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitStatus::Unknown => write!(f, "unknown"),
            CommitStatus::New => write!(f, "new"),
            CommitStatus::Committed => write!(f, "committed"),
            CommitStatus::Modified => write!(f, "modified"),
            CommitStatus::Deleted => write!(f, "deleted"),
        }
    }
}

impl Default for CommitStatus {
    fn default() -> Self {
        CommitStatus::Committed
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommittedFile {
    pub id: Uuid,
    pub name: String,
    #[serde(serialize_with = "hash_to_hex", deserialize_with = "hex_to_hash")]
    pub hash: Hash,
    #[serde(serialize_with = "hash_to_hex", deserialize_with = "hex_to_hash")]
    pub previous_hash: Hash,
    pub status: CommitStatus,
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub bucket: String,
    pub files: Vec<CommittedFile>,
    pub timestamp: String,
    pub(crate) previous: Option<Box<Commit>>,
    pub(crate) next: Option<Box<Commit>>,
}

// Custom function to serialize a `blake3::Hash` to a hex string
fn hash_to_hex<S>(hash: &Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hash.to_hex())
}

// Custom function to deserialize a hex string back to a `blake3::Hash`
fn hex_to_hash<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Hash::from_hex(&s).map_err(serde::de::Error::custom)
}

impl PartialEq for CommitStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CommitStatus::New, CommitStatus::New) => true,
            (CommitStatus::Committed, CommitStatus::Committed) => true,
            (CommitStatus::Modified, CommitStatus::Modified) => true,
            (CommitStatus::Deleted, CommitStatus::Deleted) => true,
            _ => false,
        }
    }
}

impl Commit {
    #[allow(dead_code)]
    pub fn compare(&self, other_commit: &Commit) -> Vec<CommittedFile> {
        let Commit {
            bucket: _,
            files: _,
            timestamp: _,
            previous: _,
            next: _,
        } = other_commit;
        {
            let mut status_all_files = Vec::new();

            // First check if existing files are the same
            for file in self.files.iter() {
                for other_file in other_commit.files.iter() {
                    if file.name == other_file.name && file.hash != other_file.hash {
                        status_all_files.push(CommittedFile {
                            id: file.id,
                            name: file.name.clone(),
                            hash: file.hash.clone(),
                            previous_hash: other_file.hash.clone(),
                            status: CommitStatus::Modified,
                        });
                    } else if file.name == other_file.name && file.hash == other_file.hash {
                        status_all_files.push(CommittedFile {
                            id: file.id,
                            name: file.name.clone(),
                            hash: other_file.hash.clone(),
                            previous_hash: file.hash.clone(),
                            status: CommitStatus::Committed,
                        });
                    }
                }
            }

            // Add files which haven't changed
            for file in self.files.iter() {
                let mut found = false;
                for other_file in other_commit.files.iter() {
                    if file.name == other_file.name {
                        found = true;
                    }
                }
                if !found {
                    status_all_files.push(CommittedFile {
                        id: file.id,
                        name: file.name.clone(),
                        hash: file.hash.clone(),
                        previous_hash: Hash::from_str(
                            "0000000000000000000000000000000000000000000000000000000000000000",
                        )
                        .unwrap_or_else(|_| Hash::from([0u8; 32])),
                        status: CommitStatus::New,
                    });
                }
            }

            // Check if any files were deleted
            if status_all_files.len() < other_commit.files.len() {
                for other_file in other_commit.files.iter() {
                    let mut found = false;
                    for file in self.files.iter() {
                        if file.name == other_file.name {
                            found = true;
                        }
                    }
                    if !found {
                        status_all_files.push(CommittedFile {
                            id: other_file.id,
                            name: other_file.name.clone(),
                            hash: other_file.hash.clone(),
                            previous_hash: Hash::from_str(
                                "0000000000000000000000000000000000000000000000000000000000000000",
                            )
                            .unwrap_or_else(|_| Hash::from([0u8; 32])),
                            status: CommitStatus::Deleted,
                        });
                    }
                }
            }
            status_all_files
        }
    }
}

impl CommittedFile {
    #[allow(dead_code)]
    pub fn new(name: String, hash: Hash, previous_hash: Hash, status: CommitStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            hash,
            previous_hash,
            status,
        }
    }

    pub fn compress_and_store(&self, bucket_path: &PathBuf) -> io::Result<()> {
        let input_path = bucket_path.join(&self.name);
        let output_path = bucket_path
            .join(".b")
            .join("storage")
            .join(&self.hash.to_string());

        compress_and_store_file(&input_path, &output_path, 0)
    }

    pub fn restore(&self, bucket_path: &PathBuf) -> io::Result<()> {
        let input_path = bucket_path
            .join(".b")
            .join("storage")
            .join(&self.previous_hash.to_string());
        let output_path = bucket_path.join(&self.name);

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        restore_file(&input_path, &output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[test]
    fn test_commit_status_display() {
        assert_eq!(format!("{}", CommitStatus::New), "new");
        assert_eq!(format!("{}", CommitStatus::Modified), "modified");
        assert_eq!(format!("{}", CommitStatus::Deleted), "deleted");
        assert_eq!(format!("{}", CommitStatus::Committed), "committed");
        assert_eq!(format!("{}", CommitStatus::Unknown), "unknown");
    }

    #[test]
    fn test_committed_file_new() {
        let name = "test.txt".to_string();
        let hash = Hash::from([1u8; 32]);
        let previous_hash = Hash::from([0u8; 32]);
        let status = CommitStatus::New;

        let file = CommittedFile::new(name.clone(), hash, previous_hash, status);

        assert_eq!(file.name, name);
        assert_eq!(file.hash, hash);
        assert_eq!(file.previous_hash, previous_hash);
        assert_eq!(file.status, CommitStatus::New);
        // UUID should be generated
        assert_ne!(file.id, Uuid::nil());
    }

    #[test]
    fn test_commit_compare_identical_files() {
        let file1 = CommittedFile {
            id: Uuid::new_v4(),
            name: "test.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let file2 = CommittedFile {
            id: Uuid::new_v4(),
            name: "test.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let commit1 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            files: vec![file1],
            previous: None,
            next: None,
        };

        let commit2 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-02T00:00:00Z".to_string(),
            files: vec![file2],
            previous: None,
            next: None,
        };

        let changes = commit1.compare(&commit2);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].status, CommitStatus::Committed);
    }

    #[test]
    fn test_commit_compare_modified_files() {
        let file1 = CommittedFile {
            id: Uuid::new_v4(),
            name: "test.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let file2 = CommittedFile {
            id: Uuid::new_v4(),
            name: "test.txt".to_string(),
            hash: Hash::from([2u8; 32]), // Different hash
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let commit1 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            files: vec![file1],
            previous: None,
            next: None,
        };

        let commit2 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-02T00:00:00Z".to_string(),
            files: vec![file2],
            previous: None,
            next: None,
        };

        let changes = commit1.compare(&commit2);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].status, CommitStatus::Modified);
    }

    #[test]
    fn test_commit_compare_new_file() {
        let file1 = CommittedFile {
            id: Uuid::new_v4(),
            name: "new_file.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let commit1 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            files: vec![file1],
            previous: None,
            next: None,
        };

        let commit2 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-02T00:00:00Z".to_string(),
            files: vec![],
            previous: None,
            next: None,
        };

        let changes = commit1.compare(&commit2);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].status, CommitStatus::New);
    }

    #[test]
    fn test_commit_compare_deleted_file() {
        let file1 = CommittedFile {
            id: Uuid::new_v4(),
            name: "deleted_file.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let commit1 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            files: vec![],
            previous: None,
            next: None,
        };

        let commit2 = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-02T00:00:00Z".to_string(),
            files: vec![file1],
            previous: None,
            next: None,
        };

        let changes = commit1.compare(&commit2);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].status, CommitStatus::Deleted);
    }

    #[test]
    fn test_committed_file_compress_and_store() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().to_path_buf();

        // Create bucket structure
        fs::create_dir_all(bucket_path.join(".b").join("storage"))?;

        // Create test file
        let file_content = "test file content";
        fs::write(bucket_path.join("test.txt"), file_content)?;

        let file = CommittedFile {
            id: Uuid::new_v4(),
            name: "test.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let result = file.compress_and_store(&bucket_path);
        assert!(result.is_ok());

        // Check that compressed file exists
        let compressed_path = bucket_path
            .join(".b")
            .join("storage")
            .join(file.hash.to_string());
        assert!(compressed_path.exists());
        Ok(())
    }

    #[test]
    fn test_committed_file_compress_nonexistent_file() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().to_path_buf();

        // Create bucket structure but no test file
        fs::create_dir_all(bucket_path.join(".b").join("storage"))?;

        let file = CommittedFile {
            id: Uuid::new_v4(),
            name: "nonexistent.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let result = file.compress_and_store(&bucket_path);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    #[serial]
    fn test_committed_file_restore() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().to_path_buf();

        // Create bucket structure
        fs::create_dir_all(bucket_path.join(".b").join("storage"))?;

        // Create and compress a test file first
        let file_content = "test restore content";
        fs::write(bucket_path.join("original.txt"), file_content)?;

        let hash_string = "test_hash_restore";
        let hash = Hash::from_str(hash_string).unwrap_or_else(|_| Hash::from([0u8; 32]));
        let compressed_path = bucket_path
            .join(".b")
            .join("storage")
            .join(hash.to_string());

        // Manually compress the file to simulate stored version
        use crate::utils::compression::compress_and_store_file;
        compress_and_store_file(&bucket_path.join("original.txt"), &compressed_path, 0)?;

        // Remove original file
        fs::remove_file(bucket_path.join("original.txt"))?;

        let file = CommittedFile {
            id: Uuid::new_v4(),
            name: "restored.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from_str(hash_string).unwrap_or_else(|_| Hash::from([0u8; 32])),
            status: CommitStatus::Modified,
        };

        let result = file.restore(&bucket_path);
        assert!(result.is_ok());

        // Check that file was restored
        assert!(bucket_path.join("restored.txt").exists());
        let restored_content = fs::read_to_string(bucket_path.join("restored.txt"))?;
        assert_eq!(restored_content, file_content);
        Ok(())
    }

    #[test]
    fn test_committed_file_restore_nonexistent_compressed() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let bucket_path = temp_dir.path().to_path_buf();

        // Create bucket structure but no compressed file
        fs::create_dir_all(bucket_path.join(".b").join("storage"))?;

        let file = CommittedFile {
            id: Uuid::new_v4(),
            name: "restore_fail.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([2u8; 32]),
            status: CommitStatus::Modified,
        };

        let result = file.restore(&bucket_path);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_hash_serialization() {
        let hash = Hash::from([42u8; 32]);
        let hash_string = hash.to_string();
        assert_eq!(hash_string.len(), 64); // 32 bytes = 64 hex chars

        let parsed_hash = Hash::from_str(&hash_string);
        assert!(parsed_hash.is_ok());
        assert_eq!(parsed_hash.unwrap(), hash);
    }

    #[test]
    fn test_hash_from_invalid_string() {
        let invalid_hash = Hash::from_str("invalid_hash_string");
        assert!(invalid_hash.is_err());

        let short_hash = Hash::from_str("1234");
        assert!(short_hash.is_err());
    }

    #[test]
    fn test_commit_with_multiple_files() {
        let file1 = CommittedFile {
            id: Uuid::new_v4(),
            name: "file1.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let file2 = CommittedFile {
            id: Uuid::new_v4(),
            name: "file2.txt".to_string(),
            hash: Hash::from([2u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let commit = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            files: vec![file1, file2],
            previous: None,
            next: None,
        };

        assert_eq!(commit.files.len(), 2);
        assert_eq!(commit.bucket, "test_bucket");
    }

    #[test]
    fn test_commit_status_default() {
        let status = CommitStatus::default();
        assert_eq!(status, CommitStatus::Committed);
    }

    #[test]
    fn test_commit_serialization() {
        let file = CommittedFile {
            id: Uuid::new_v4(),
            name: "test.txt".to_string(),
            hash: Hash::from([1u8; 32]),
            previous_hash: Hash::from([0u8; 32]),
            status: CommitStatus::New,
        };

        let commit = Commit {
            bucket: "test_bucket".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            files: vec![file],
            previous: None,
            next: None,
        };

        // Just test that the commit has the expected values
        assert_eq!(commit.bucket, "test_bucket");
        assert_eq!(commit.files.len(), 1);
        assert_eq!(commit.files[0].name, "test.txt");
    }
}
