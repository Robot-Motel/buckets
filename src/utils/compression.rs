use std::{fs::File, io::{self}, path::PathBuf};

use zstd::{stream::{copy_decode, copy_encode}};

pub fn compress_and_store_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    compression_level: i32,
) -> io::Result<()> {
    let input_file = File::open(input_path)
        .map_err(|_e| io::Error::new(io::ErrorKind::NotFound, 
            format!("Failed to open input file: {}", input_path.display())))?;
    let mut output_file = File::create(output_path)?;

    copy_encode(&input_file, &mut output_file, compression_level)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
            format!("Failed to compress file from {} to {}: {}", 
                input_path.display(), output_path.display(), e)))?;

    Ok(())
}

pub fn restore_file(input_path: &PathBuf, output_path: &PathBuf) -> io::Result<()> {
    let input = File::open(input_path)?;
    let output = File::create(output_path)?;
    copy_decode(input, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{fs, io::Write};
    use tempfile::tempdir;

    #[test]
    fn test_compress_and_store_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("input.txt");
        let result_path = dir.path().join("result.txt");
        let output_path = dir.path().join("output.zst");

        // Create a test file
        fs::write(&input_path, "Hello, world!").expect("Failed to write test file");

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).expect("Failed to compress file");

        // Check if the compressed file exists
        assert!(output_path.exists());

        // uncompress the file
        let mut decoder = zstd::stream::Decoder::new(File::open(&output_path).expect("Failed to open compressed file"))
            .expect("Failed to create decoder");
        let mut writer = File::create(&result_path).expect("Failed to create result file");
        io::copy(&mut decoder, &mut writer).expect("Failed to copy decoded data");
        decoder.finish();

        
        // Check if the uncompressed file exists
        assert!(result_path.exists());
        let content = fs::read_to_string(&result_path).expect("Failed to read result file");
        assert_eq!(content, "Hello, world!");

    }

    #[test]
    fn test_restore_file() {
        let mut dir = tempdir().expect("Failed to create temp dir");
        dir.disable_cleanup(true);
        log::info!("dir: {:?}", dir.path());
        let bucket_path = dir.path().to_path_buf();
        let compressed_file_path = bucket_path.join("input.zst");
        let restored_file_path = bucket_path.join("output.txt");

        // Create a test file
        fs::write(&restored_file_path, "Hello, world!").expect("Failed to write test file");

        let writer = File::create(&compressed_file_path).expect("Failed to create compressed file");

        let mut encoder = zstd::stream::Encoder::new(&writer, 7).expect("Failed to create encoder");
        io::copy(&mut File::open(&restored_file_path).expect("Failed to open source file"), &mut encoder)
            .expect("Failed to copy to encoder");
        encoder.finish().expect("Failed to finish encoding");
      

        // remove the original file
        fs::remove_file(&restored_file_path).expect("Failed to remove original file");

        // Restore the file
        restore_file(&compressed_file_path, &restored_file_path)
            .expect("Failed to restore file");
        log::info!("File restored successfully");

        // Check if the restored file exists and contains the expected content
        assert!(restored_file_path.exists());
        let content = fs::read_to_string(&restored_file_path).expect("Failed to read restored file");
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_compress_and_store_large_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("input.txt");
        let result_path = dir.path().join("result.txt");
        let output_path = dir.path().join("output.zst");

        // Create a large test file
        let mut file = File::create(&input_path).expect("Failed to create test file");
        let buffer = vec![0; 1024 * 1024 * 10]; // 10MB
        file.write_all(&buffer).expect("Failed to write test data");

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).expect("Failed to compress large file");

        // Check if the compressed file exists
        assert!(output_path.exists());

        // uncompress the file
        let mut decoder = zstd::stream::Decoder::new(File::open(&output_path).expect("Failed to open compressed file"))
            .expect("Failed to create decoder");
        let mut writer = File::create(&result_path).expect("Failed to create result file");
        io::copy(&mut decoder, &mut writer).expect("Failed to copy decoded data");   
        decoder.finish();

        // Check if the uncompressed file exists
        assert!(result_path.exists());

    }

    #[test]
    fn test_restore_large_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("input.txt");
        let result_path = dir.path().join("result.txt");
        let output_path = dir.path().join("output.zst");

        // Create a large test file
        let mut file = File::create(&input_path).expect("Failed to create test file");
        let buffer = vec![0; 1024 * 1024 * 10]; // 10MB
        file.write_all(&buffer).expect("Failed to write test data");

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).expect("Failed to compress large file");

        // Restore the file
        restore_file(&output_path, &result_path).expect("Failed to restore large file");

        // Check if the restored file exists
        assert!(result_path.exists());
    }

    #[test]
    fn test_compress_nonexistent_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let nonexistent_path = dir.path().join("does_not_exist.txt");
        let output_path = dir.path().join("output.zst");

        // Test compression of non-existent file
        let result = compress_and_store_file(&nonexistent_path, &output_path, 3);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::NotFound);
        }
    }

    #[test]
    fn test_compress_to_invalid_directory() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("input.txt");
        let invalid_output_path = dir.path().join("nonexistent_dir").join("output.zst");

        // Create input file
        fs::write(&input_path, "test content").expect("Failed to write input file");

        // Test compression to invalid output directory
        let result = compress_and_store_file(&input_path, &invalid_output_path, 3);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::NotFound);
        }
    }

    #[test] 
    fn test_restore_nonexistent_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let nonexistent_compressed = dir.path().join("does_not_exist.zst");
        let output_path = dir.path().join("output.txt");

        // Test restoration of non-existent compressed file
        let result = restore_file(&nonexistent_compressed, &output_path);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::NotFound);
        }
    }

    #[test]
    fn test_restore_invalid_compressed_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let fake_compressed = dir.path().join("fake.zst");
        let output_path = dir.path().join("output.txt");

        // Create fake compressed file (not actually compressed)
        fs::write(&fake_compressed, "this is not compressed data").expect("Failed to write fake file");

        // Test restoration of invalid compressed file
        let result = restore_file(&fake_compressed, &output_path);
        assert!(result.is_err());
        // Should fail during decompression
    }

    #[test]
    fn test_restore_to_readonly_directory() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("input.txt");
        let compressed_path = dir.path().join("compressed.zst");
        
        // Create readonly subdirectory
        let readonly_dir = dir.path().join("readonly");
        fs::create_dir_all(&readonly_dir).expect("Failed to create readonly dir");
        
        // Set readonly permissions (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).expect("Failed to get metadata").permissions();
            perms.set_mode(0o444); // readonly
            fs::set_permissions(&readonly_dir, perms).expect("Failed to set permissions");
        }

        // Create and compress test file
        fs::write(&input_path, "test content").expect("Failed to write input file");
        compress_and_store_file(&input_path, &compressed_path, 3).expect("Failed to compress");

        let readonly_output = readonly_dir.join("output.txt");
        
        // Test restoration to readonly directory (may fail on some systems)
        let result = restore_file(&compressed_path, &readonly_output);
        
        // Restore directory permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).expect("Failed to get metadata").permissions();
            perms.set_mode(0o755); // writable
            fs::set_permissions(&readonly_dir, perms).expect("Failed to restore permissions");
        }
        
        // On some systems this might succeed, on others it might fail with PermissionDenied
        if result.is_err() {
            if let Err(e) = result {
                assert!(e.kind() == io::ErrorKind::PermissionDenied || e.kind() == io::ErrorKind::NotFound);
            }
        }
    }

    #[test]
    fn test_compress_empty_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("empty.txt");
        let output_path = dir.path().join("empty.zst");
        let restored_path = dir.path().join("restored.txt");

        // Create empty file
        File::create(&input_path).expect("Failed to create empty file");

        // Compress empty file
        compress_and_store_file(&input_path, &output_path, 3).expect("Failed to compress empty file");
        assert!(output_path.exists());

        // Restore empty file
        restore_file(&output_path, &restored_path).expect("Failed to restore empty file");
        assert!(restored_path.exists());
        
        let restored_content = fs::read(&restored_path).expect("Failed to read restored file");
        assert!(restored_content.is_empty());
    }

    #[test]
    fn test_compression_with_different_levels() {
        let dir = tempdir().expect("Failed to create temp dir");
        let input_path = dir.path().join("input.txt");
        let test_content = "This is test content for compression level testing.".repeat(100);
        
        fs::write(&input_path, &test_content).expect("Failed to write test file");

        // Test different compression levels
        for level in [0, 1, 3, 9, 22] { // zstd supports levels 1-22, 0 is default
            let output_path = dir.path().join(format!("output_level_{}.zst", level));
            let restored_path = dir.path().join(format!("restored_level_{}.txt", level));

            // Compress with specific level
            let result = compress_and_store_file(&input_path, &output_path, level);
            assert!(result.is_ok(), "Failed to compress with level {}", level);
            assert!(output_path.exists());

            // Restore and verify content
            restore_file(&output_path, &restored_path).expect("Failed to restore");
            let restored_content = fs::read_to_string(&restored_path).expect("Failed to read restored");
            assert_eq!(restored_content, test_content);
        }
    }

    #[test]
    fn test_compression_roundtrip_various_data() {
        let dir = tempdir().expect("Failed to create temp dir");
        
        let binary_data = (0u8..=255u8).cycle().take(1000).collect::<Vec<u8>>();
        let binary_string = String::from_utf8_lossy(&binary_data);
        let repeated_string = "A".repeat(10000);
        
        let test_cases = vec![
            ("text", "Hello, world! This is a test string with various characters: 你好世界 🌍"),
            ("binary", binary_string.as_ref()),
            ("repeated", repeated_string.as_str()),
            ("json", r#"{"name": "test", "value": 42, "items": [1, 2, 3, {"nested": true}]}"#),
            ("whitespace", "   \t\n\r  \n\n  \t  "),
        ];

        for (name, content) in test_cases {
            let input_path = dir.path().join(format!("input_{}.txt", name));
            let compressed_path = dir.path().join(format!("compressed_{}.zst", name));
            let restored_path = dir.path().join(format!("restored_{}.txt", name));

            // Write test content
            fs::write(&input_path, content).expect("Failed to write test content");

            // Compress
            compress_and_store_file(&input_path, &compressed_path, 3)
                .expect("Failed to compress test content");
            assert!(compressed_path.exists());

            // Restore
            restore_file(&compressed_path, &restored_path)
                .expect("Failed to restore test content");
            assert!(restored_path.exists());

            // Verify content matches
            let restored_content = fs::read_to_string(&restored_path)
                .expect("Failed to read restored content");
            assert_eq!(restored_content, content, "Content mismatch for test case: {}", name);
        }
    }
}
