use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use blake3::Hash;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
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
            CommitStatus::Unknown => write!(f, "Unknown"),
            CommitStatus::New => write!(f, "New"),
            CommitStatus::Committed => write!(f, "Committed"),
            CommitStatus::Modified => write!(f, "Modified"),
            CommitStatus::Deleted => write!(f, "Deleted"),
        }
    }
}

impl Default for CommitStatus {
    fn default() -> Self {
        CommitStatus::Committed
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommittedFile {
    pub id: Uuid,
    pub name: String,
    #[serde(serialize_with = "hash_to_hex", deserialize_with = "hex_to_hash")]
    pub hash: Hash,
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
    pub fn compare(&self, other_commit: &Commit) -> Option<Vec<CommittedFile>> {
        match other_commit {
            Commit {
                bucket: _,
                files: _,
                timestamp: _,
                previous: _,
                next: _,
            } => {
                let mut changes = Vec::new();

                // First check if existing files are the same
                for file in self.files.iter() {
                    for other_file in other_commit.files.iter() {
                        if file.name == other_file.name && file.hash != other_file.hash {
                            changes.push(CommittedFile {
                                id: file.id,
                                name: file.name.clone(),
                                hash: file.hash.clone(),
                                status: CommitStatus::Modified,
                            });
                        } else if file.name == other_file.name && file.hash == other_file.hash {
                            changes.push(CommittedFile {
                                id: file.id,
                                name: file.name.clone(),
                                hash: other_file.hash.clone(),
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
                        changes.push(CommittedFile {
                            id: file.id,
                            name: file.name.clone(),
                            hash: file.hash.clone(),
                            status: CommitStatus::New,
                        });
                    }
                }

                // Check if any files were deleted
                if changes.len() < other_commit.files.len() {
                    for other_file in other_commit.files.iter() {
                        let mut found = false;
                        for file in self.files.iter() {
                            if file.name == other_file.name {
                                found = true;
                            }
                        }
                        if !found {
                            changes.push(CommittedFile {
                                id: other_file.id,
                                name: other_file.name.clone(),
                                hash: other_file.hash.clone(),
                                status: CommitStatus::Deleted,
                            });
                        }
                    }
                }
                return Some(changes);
            }

        }

    }
}

