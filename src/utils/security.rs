use crate::errors::BucketError;
use std::path::{Path, PathBuf, Component};

/// Maximum allowed path depth to prevent excessive recursion
const MAX_PATH_DEPTH: usize = 100;

/// Maximum file name length to prevent buffer overflow attacks
const MAX_FILENAME_LENGTH: usize = 255;

/// Validates and normalizes a path to prevent path traversal attacks
/// 
/// This function:
/// - Resolves path components to prevent directory traversal
/// - Validates path depth to prevent excessive recursion
/// - Checks for dangerous path components
/// - Ensures paths stay within allowed boundaries
pub fn validate_and_canonicalize_path(path: &Path, base_dir: Option<&Path>) -> Result<PathBuf, BucketError> {
    // Reject paths with null bytes
    if path.as_os_str().to_string_lossy().contains('\0') {
        return Err(BucketError::SecurityError(
            "Path contains null bytes".to_string()
        ));
    }

    // Check for excessively long file names
    if let Some(filename) = path.file_name() {
        if filename.len() > MAX_FILENAME_LENGTH {
            return Err(BucketError::SecurityError(
                format!("Filename too long (max {} characters)", MAX_FILENAME_LENGTH)
            ));
        }
    }

    // Normalize path by resolving components
    let normalized = normalize_path_components(path)?;
    
    // Check path depth
    if normalized.components().count() > MAX_PATH_DEPTH {
        return Err(BucketError::SecurityError(
            format!("Path too deep (max {} components)", MAX_PATH_DEPTH)
        ));
    }

    // If a base directory is provided, ensure the path is within it
    if let Some(base) = base_dir {
        let base_canonical = base.canonicalize().map_err(|e| {
            BucketError::PathValidationError(format!("Cannot canonicalize base path: {}", e))
        })?;
        
        let full_path = if normalized.is_relative() {
            base_canonical.join(&normalized)
        } else {
            normalized.clone()
        };
        
        // Attempt to canonicalize if the path exists
        let canonical_path = if full_path.exists() {
            full_path.canonicalize().map_err(|e| {
                BucketError::PathValidationError(format!("Cannot canonicalize path: {}", e))
            })?
        } else {
            // For non-existent paths, canonicalize the parent and join the filename
            let parent = full_path.parent().unwrap_or(&full_path);
            if parent.exists() {
                let canonical_parent = parent.canonicalize().map_err(|e| {
                    BucketError::PathValidationError(format!("Cannot canonicalize parent path: {}", e))
                })?;
                if let Some(filename) = full_path.file_name() {
                    canonical_parent.join(filename)
                } else {
                    canonical_parent
                }
            } else {
                full_path
            }
        };
        
        // Ensure the canonical path is still within the base directory
        if !canonical_path.starts_with(&base_canonical) {
            return Err(BucketError::SecurityError(
                "Path traversal attempt detected".to_string()
            ));
        }
        
        Ok(canonical_path)
    } else {
        Ok(normalized)
    }
}

/// Normalizes path by resolving . and .. components manually
fn normalize_path_components(path: &Path) -> Result<PathBuf, BucketError> {
    let mut components = Vec::new();
    
    for component in path.components() {
        match component {
            Component::Normal(name) => {
                // Check for dangerous filenames
                let name_str = name.to_string_lossy();
                if is_dangerous_filename(&name_str) {
                    return Err(BucketError::SecurityError(
                        format!("Dangerous filename detected: {}", name_str)
                    ));
                }
                components.push(component);
            }
            Component::ParentDir => {
                if components.is_empty() {
                    return Err(BucketError::SecurityError(
                        "Path attempts to traverse above root".to_string()
                    ));
                }
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            Component::CurDir => {
                // Skip current directory references
                continue;
            }
            Component::RootDir | Component::Prefix(_) => {
                components.push(component);
            }
        }
    }
    
    let mut result = PathBuf::new();
    for component in components {
        result.push(component);
    }
    
    Ok(result)
}

/// Check for dangerous filenames that could be used for attacks
fn is_dangerous_filename(name: &str) -> bool {
    // Check for Windows reserved names
    const WINDOWS_RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
    ];
    
    let upper_name = name.to_uppercase();
    if WINDOWS_RESERVED.contains(&upper_name.as_str()) {
        return true;
    }
    
    // Check for names that end with reserved names + extension
    for reserved in WINDOWS_RESERVED {
        if upper_name.starts_with(reserved) && upper_name.len() > reserved.len() {
            let remainder = &upper_name[reserved.len()..];
            if remainder.starts_with('.') {
                return true;
            }
        }
    }
    
    // Check for control characters and other dangerous characters
    for ch in name.chars() {
        if ch.is_control() || ch == '\0' {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[test]
    fn test_validate_and_canonicalize_path_normal() {
        let temp_dir = tempdir().unwrap();
        let test_path = PathBuf::from("test/file.txt");
        
        let result = validate_and_canonicalize_path(&test_path, Some(temp_dir.path()));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_and_canonicalize_path_traversal_attack() {
        let temp_dir = tempdir().unwrap();
        let malicious_path = PathBuf::from("../../../etc/passwd");
        
        let result = validate_and_canonicalize_path(&malicious_path, Some(temp_dir.path()));
        assert!(result.is_err());
        
        match result {
            Err(BucketError::SecurityError(msg)) => {
                assert!(msg.contains("traversal") || msg.contains("above root"));
            }
            Err(e) => {
                // Print the actual error for debugging
                println!("Actual error: {:?}", e);
                panic!("Expected SecurityError for path traversal, got: {:?}", e);
            }
            Ok(_) => panic!("Expected error for path traversal attack"),
        }
    }
    
    #[test]
    fn test_validate_and_canonicalize_path_null_bytes() {
        let temp_dir = tempdir().unwrap();
        let path_with_null = PathBuf::from("file\0.txt");
        
        let result = validate_and_canonicalize_path(&path_with_null, Some(temp_dir.path()));
        assert!(result.is_err());
        
        if let Err(BucketError::SecurityError(msg)) = result {
            assert!(msg.contains("null bytes"));
        } else {
            panic!("Expected SecurityError for null bytes");
        }
    }
    
    #[test]
    fn test_is_dangerous_filename() {
        assert!(is_dangerous_filename("CON"));
        assert!(is_dangerous_filename("con"));
        assert!(is_dangerous_filename("NUL.txt"));
        assert!(is_dangerous_filename("file\x00name"));
        assert!(!is_dangerous_filename("normal_file.txt"));
    }
    
    #[test]
    fn test_normalize_path_components() {
        let path = PathBuf::from("a/./b/../c");
        let result = normalize_path_components(&path).unwrap();
        assert_eq!(result, PathBuf::from("a/c"));
        
        let dangerous_path = PathBuf::from("../../etc/passwd");
        let result = normalize_path_components(&dangerous_path);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_path_exists() {
        let temp_dir = tempdir().unwrap();
        let existing_path = temp_dir.path().join("test.txt");
        fs::write(&existing_path, "test").unwrap();
        
        assert!(validate_path_exists(&existing_path).is_ok());
        
        let non_existing_path = temp_dir.path().join("does_not_exist.txt");
        assert!(validate_path_exists(&non_existing_path).is_err());
    }
    
    #[test]
    fn test_secure_delete_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("to_delete.txt");
        fs::write(&test_file, "sensitive data").unwrap();
        
        assert!(test_file.exists());
        assert!(secure_delete_file(&test_file).is_ok());
        assert!(!test_file.exists());
    }
    
    /// Securely delete a file by overwriting it before removal
    pub fn secure_delete_file(path: &Path) -> Result<(), BucketError> {
        use std::fs::OpenOptions;
        use std::io::{Write, Seek, SeekFrom};
        
        if !path.is_file() {
            return Err(BucketError::PathValidationError(
                "Path is not a file".to_string()
            ));
        }
        
        // Get file size
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        
        // Overwrite file contents with random data
        let mut file = OpenOptions::new()
            .write(true)
            .open(path)?;
        
        // Simple overwrite with zeros (in production, use random data)
        let zeros = vec![0u8; 4096];
        let mut remaining = file_size;
        
        file.seek(SeekFrom::Start(0))?;
        while remaining > 0 {
            let chunk_size = std::cmp::min(remaining, zeros.len() as u64) as usize;
            file.write_all(&zeros[..chunk_size])?;
            remaining -= chunk_size as u64;
        }
        
        file.sync_all()?;
        drop(file);
        
        // Now remove the file
        fs::remove_file(path)?;
        
        Ok(())
    }

    /// Validate that a path exists and is accessible
    pub fn validate_path_exists(path: &Path) -> Result<(), BucketError> {
        if !path.exists() {
            return Err(BucketError::PathValidationError(
                format!("Path does not exist: {}", path.display())
            ));
        }
        
        // Try to read metadata to ensure we have access
        fs::metadata(path).map_err(|e| {
            BucketError::SecurityError(format!("Cannot access path {}: {}", path.display(), e))
        })?;
        
        Ok(())
    }

}