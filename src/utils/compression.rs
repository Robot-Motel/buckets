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
}
