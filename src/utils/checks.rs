use std::fs;
use std::path::{Path, PathBuf};

/// Searches for a directory with the given name in the parent directories.
///
/// # Arguments
///
/// * `start_path` - The path to start the search from.
/// * `target_dir_name` - The name of the directory to search for.
///
/// # Returns
///
/// Returns `Some(PathBuf)` containing the path to the found directory or `None` if not found.
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

pub fn is_valid_bucket_repo(dir_path: &Path) -> bool {
    let buckets_repo_path = find_directory_in_parents(dir_path, ".buckets");
    println!("{:?}", buckets_repo_path);
    match buckets_repo_path {
        Some(path) => is_valid_repo_config(&path),
        None => false,
    }
}

#[allow(dead_code)]
pub fn is_valid_bucket(bucket_path: &Path) -> bool {
    let buckets_path = find_directory_in_parents(bucket_path, ".b");
    match buckets_path {
        Some(path) => has_valid_bucket_info(&path),
        None => false,
    }
}

#[allow(dead_code)]
fn has_valid_bucket_info(bucket_path: &PathBuf) -> bool {
    let info_path = bucket_path.join(".b").join("info");
    if info_path.exists() && info_path.is_file() {
        return true;
    }
    false
}

#[allow(dead_code)]
pub fn db_location(dir_path: &Path) -> PathBuf {
    let buckets_repo_path = find_directory_in_parents(dir_path, ".buckets").unwrap();
    buckets_repo_path.join("buckets.db")
}


pub fn is_valid_repo_config(dir_path: &Path) -> bool {
    let config_path = dir_path.join("config");
    if config_path.is_file() {
        return true;
    }
    false
}

#[allow(dead_code)]
pub fn is_valid_bucket_info(dir_path: &Path) -> bool {
    let config_path = dir_path.join("info");
    if config_path.is_file() {
        return true;
    }
    false
}

/// Searches for a bucket repository directory in the parent directories.
///
/// # Arguments
///
/// * `dir_path` - The path to start the search from.
///
/// # Returns
///
/// Returns `Some(PathBuf)` containing the path to the found bucket repository directory or `None` if not found.
///
#[allow(dead_code)]
pub fn find_bucket_repo(dir_path: &Path) -> Option<PathBuf> {
    match find_directory_in_parents(dir_path, ".buckets") {
        Some(path) => Some(path),
        None => None,
    }
}



#[cfg(test)]
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, File};

    #[test]
    fn test_find_directory_in_parents() {
        let temp_dir = tempdir().unwrap();
        let target_dir_name = "target_directory";

        // Create a nested directory structure
        let nested_dir_path = temp_dir.path().join("a/b/c/d/e");
        create_dir_all(&nested_dir_path).unwrap();

        // Create the target directory
        let target_dir_path = temp_dir.path().join("a/target_directory");
        create_dir_all(&target_dir_path).unwrap();

        // Start the search from the deepest directory
        let start_path = nested_dir_path;

        // Perform the test
        let result = find_directory_in_parents(&start_path, target_dir_name);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), target_dir_path);
    }

    #[test]
    fn test_find_directory_in_current_dir() {
        let temp_dir = tempdir().unwrap();
        let target_dir_name = "target_directory";

        // Create a nested directory structure
        let nested_dir_path = temp_dir.path().join("a");
        create_dir_all(&nested_dir_path).unwrap();

        // Create the target directory
        let target_dir_path = temp_dir.path().join("a/target_directory");
        create_dir_all(&target_dir_path).unwrap();

        // Start the search from the deepest directory
        let start_path = nested_dir_path;

        // Perform the test
        let result = find_directory_in_parents(&start_path, target_dir_name);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), target_dir_path);
    }

    #[test]
    fn test_find_directory_in_parents_not_found() {
        let temp_dir = tempdir().unwrap();
        let target_dir_name = "target_directory";

        // Create a nested directory structure
        let nested_dir_path = temp_dir.path().join("a/b/c/d/e");
        create_dir_all(&nested_dir_path).unwrap();

        // Start the search from the deepest directory
        let start_path = nested_dir_path;

        // Perform the test
        let result = find_directory_in_parents(&start_path, target_dir_name);
        assert!(result.is_none());
    }

    #[test]
    fn test_valid_bucket_repo() {
        let temp_dir = tempdir().unwrap();
        let buckets_dir = temp_dir.path().join(".buckets");
        fs::create_dir(&buckets_dir).unwrap();
        let config_file = buckets_dir.join("config");
        File::create(&config_file).unwrap();

        assert!(is_valid_bucket_repo(temp_dir.path()));
    }

    #[test]
    fn test_invalid_bucket_repo() {
        let temp_dir = tempdir().unwrap();
        assert!(!is_valid_bucket_repo(temp_dir.path()));
    }

    #[test]
    fn test_valid_repo_config() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("config");
        File::create(&config_file).unwrap();

        assert!(is_valid_repo_config(temp_dir.path()));
    }

    #[test]
    fn test_invalid_repo_config() {
        let temp_dir = tempdir().unwrap();
        assert!(!is_valid_repo_config(temp_dir.path()));
    }
}
