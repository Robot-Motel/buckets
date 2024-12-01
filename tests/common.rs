use std::path::PathBuf;
use tempfile::tempdir;

#[allow(dead_code)]
pub fn get_test_dir() -> PathBuf {
    let temp_dir = match std::env::var("TEST_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => tempdir().unwrap().into_path(),
    };
    temp_dir
}