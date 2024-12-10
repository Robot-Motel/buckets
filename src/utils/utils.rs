use std::{env, fs, io};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use blake3::{Hash, Hasher};
use duckdb::Connection;
use walkdir::{DirEntry, WalkDir};
use crate::errors::BucketError;

#[allow(dead_code)]
pub fn delete_and_create_tmp_dir(bucket_path: &PathBuf) -> Result<PathBuf, BucketError> {
    let tmp_bucket_path = bucket_path.join(".b").join("tmp");
    fs::remove_dir_all(&tmp_bucket_path).unwrap_or_default();
    fs::create_dir_all(&tmp_bucket_path)?;
    Ok(tmp_bucket_path)
}

pub(crate) fn find_files_excluding_top_level_b(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| is_not_in_dir(entry, &dir, ".b"))
        .filter_map(|entry| make_relative_path(entry.path(), &dir))
        .collect()
}

fn is_not_in_dir(entry: &DirEntry, root_dir: &Path, excluded_dir: &str) -> bool {
    let is_top_level_ex_dir = entry.depth() == 1 && entry.file_name() == excluded_dir;
    let is_inside_top_level_ex_dir = entry.path().starts_with(&root_dir.join(excluded_dir));

    entry.file_type().is_file() && !is_top_level_ex_dir && !is_inside_top_level_ex_dir
}

fn make_relative_path(path: &Path, base: &Path) -> Option<PathBuf> {
    path.strip_prefix(base).ok().map(PathBuf::from)
}

pub(crate) fn hash_file<P: AsRef<Path>>(path: P) -> io::Result<Hash> {
    let mut file = File::open(path)?;
    let mut hasher = Hasher::new();
    let mut buffer = [0; 1024]; // Buffer for reading chunks

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break; // End of file
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize())
}

pub fn find_directory_in_parents(start_path: &Path, target_dir_name: &str) -> Option<PathBuf> {
    let mut current_path = start_path;

    let potential_target = current_path.join(target_dir_name);
    if potential_target.is_dir() && fs::metadata(&potential_target).is_ok() {
        return Some(potential_target);
    }

    while let Some(parent) = current_path.parent() {
        let potential_target = parent.join(target_dir_name);
        if potential_target.is_dir() && fs::metadata(&potential_target).is_ok() {
            return Some(potential_target);
        }
        current_path = parent;
    }

    None
}

pub fn find_bucket_repo(dir_path: &Path) -> Option<PathBuf> {
    match find_directory_in_parents(dir_path, ".buckets") {
        Some(path) => Some(path),
        None => None,
    }
}

pub fn connect_to_db() -> Result<Connection, BucketError> {
    let path = match find_directory_in_parents(env::current_dir()?.as_path(), ".buckets") {
        Some(path) => path,
        None => return Err(BucketError::NotInBucketsRepo),
    };

    let db_location = db_location(path.as_path());
    match Connection::open(db_location) {
        Ok(conn) => {
            return Ok(conn);
        },
        Err(e) => {
            return Err(BucketError::DuckDB(e));
        },
    }
}

pub fn db_location(dir_path: &Path) -> PathBuf {
    let buckets_repo_path = find_directory_in_parents(dir_path, ".buckets").unwrap();
    buckets_repo_path.join("buckets.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::tempdir;

    #[test]
    fn test_delete_and_create_tmp_dir() {
        let temp_dir = tempdir().unwrap();
        let bucket_tmp_path = temp_dir.path().join("bucket").join(".b").join("tmp");
        create_dir_all(&bucket_tmp_path).unwrap();

        let bucket_path = temp_dir.path().join("bucket");
        let result = delete_and_create_tmp_dir(&bucket_path);
        assert!(result.is_ok());
        assert!(bucket_path.join(".b").join("tmp").exists());
    }

    #[test]
    fn test_delete_and_create_tmp_dir_not_exist() {
        let temp_dir = tempdir().unwrap();
        let bucket_b_path = temp_dir.path().join("bucket").join(".b");
        create_dir_all(&bucket_b_path).unwrap();

        let bucket_path = temp_dir.path().join("bucket");
        let result = delete_and_create_tmp_dir(&bucket_path);
        assert!(result.is_ok());
        assert!(bucket_path.join(".b").join("tmp").exists());
    }

    #[test]
    fn test_is_not_in_dir() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();

        // Create test files and directories
        // ./file1.txt
        // ./.b/file2.txt
        // ./subdir/file3.txt
        // ./subdir/subsubdir/file4.txt
        // ./.b/subsubdir/file5.txt
        fs::create_dir_all(dir_path.join(".b").join("subsubdir")).unwrap();
        fs::create_dir_all(dir_path.join("subdir").join("subsubdir")).unwrap();
        fs::write(dir_path.join("file1.txt"), b"file1").unwrap();
        fs::write(dir_path.join(".b").join("file2.txt"), b"file2").unwrap();
        fs::write(dir_path.join("subdir").join("file3.txt"), b"file3").unwrap();
        fs::write(dir_path.join("subdir").join("subsubdir").join("file4.txt"), b"file4").unwrap();
        fs::write(dir_path.join(".b").join("subsubdir").join("file5.txt"), b"file5").unwrap();

        let root_dir = dir_path;

        let entry_file1 = WalkDir::new(dir_path.join("file1.txt")).into_iter().next().unwrap().unwrap();
        let entry_file2 = WalkDir::new(dir_path.join(".b").join("file2.txt")).into_iter().next().unwrap().unwrap();
        let entry_file3 = WalkDir::new(dir_path.join("subdir").join("file3.txt")).into_iter().next().unwrap().unwrap();
        let entry_file4 = WalkDir::new(dir_path.join("subdir").join("subsubdir").join("file4.txt")).into_iter().next().unwrap().unwrap();
        let entry_file5 = WalkDir::new(dir_path.join(".b").join("subsubdir").join("file5.txt")).into_iter().next().unwrap().unwrap();

        assert!(is_not_in_dir(&entry_file1, root_dir, ".b"));
        assert!(!is_not_in_dir(&entry_file2, root_dir, ".b"));
        assert!(is_not_in_dir(&entry_file3, root_dir, ".b"));
        assert!(is_not_in_dir(&entry_file4, root_dir, ".b"));
        assert!(!is_not_in_dir(&entry_file5, root_dir, ".b"));
        assert!(is_not_in_dir(&entry_file5, root_dir, "other"));

        temp_dir.close().unwrap();
    }
    #[test]
    fn test_make_relative_path() {
        let base = Path::new("/base/dir");
        let path = Path::new("/base/dir/subdir/file.txt");
        let result = make_relative_path(path, base);
        assert_eq!(result, Some(PathBuf::from("subdir/file.txt")));

        let path_outside_base = Path::new("/other/dir/file.txt");
        let result = make_relative_path(path_outside_base, base);
        assert_eq!(result, None);
    }
}
