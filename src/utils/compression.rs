use std::{fs::File, io::{self}, path::PathBuf};

use zstd::{stream::{copy_decode, copy_encode}};

pub fn compress_and_store_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    compression_level: i32,
) -> io::Result<()> {
    let input_file = File::open(input_path).map_err(|e| {
        panic!("Failed to open input file: {} \nError: {}", input_path.display(), e.to_string());
    }).unwrap();
    let mut output_file = File::create(output_path)?;

    let _ = copy_encode( &input_file, &mut output_file, compression_level).map_err(|e| {
        panic!("Failed to compress and store file: {} \nInput path: {} \nOutput path: {}", e, input_path.display(), output_path.display())
    });

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
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let result_path = dir.path().join("result.txt");
        let output_path = dir.path().join("output.zst");

        // Create a test file
        fs::write(&input_path, "Hello, world!").unwrap();

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).unwrap();

        // Check if the compressed file exists
        assert!(output_path.exists());

        // uncompress the file
        let mut decoder = zstd::stream::Decoder::new(File::open(&output_path).unwrap()).unwrap();
        let mut writer = File::create(&result_path).unwrap();
        io::copy(&mut decoder, &mut writer).unwrap();
        decoder.finish();

        
        // Check if the uncompressed file exists
        assert!(result_path.exists());
        let content = fs::read_to_string(&result_path).unwrap();
        assert_eq!(content, "Hello, world!");

    }

    #[test]
    fn test_restore_file() {
        let mut dir = tempdir().unwrap();
        dir.disable_cleanup(true);
        log::info!("dir: {:?}", dir.path());
        let bucket_path = dir.path().to_path_buf();
        let compressed_file_path = bucket_path.join("input.zst");
        let restored_file_path = bucket_path.join("output.txt");

        // Create a test file
        fs::write(&restored_file_path, "Hello, world!").unwrap();

        let writer = File::create(&compressed_file_path).unwrap();

        let mut encoder = zstd::stream::Encoder::new(&writer, 7).unwrap();
        io::copy(&mut File::open(&restored_file_path).unwrap(), &mut encoder).unwrap();
        encoder.finish().unwrap();
      

        // remove the original file
        fs::remove_file(&restored_file_path).unwrap();

        // Restore the file, display the error if it fails
        restore_file( &compressed_file_path, &restored_file_path).unwrap_or_else(|e| {
            log::error!("Failed to restore file: {} \nError: {}", compressed_file_path.display(), e.to_string());
            panic!("Failed to restore file:{} \nError: {}", compressed_file_path.display(), e.to_string());
        });
        log::info!("File restored successfully");

        // Check if the restored file exists and contains the expected content
        assert!(restored_file_path.exists());
        let content = fs::read_to_string(&restored_file_path).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_compress_and_store_large_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let result_path = dir.path().join("result.txt");
        let output_path = dir.path().join("output.zst");

        // Create a large test file
        let mut file = File::create(&input_path).unwrap();
        let buffer = vec![0; 1024 * 1024 * 10]; // 10MB
        file.write_all(&buffer).unwrap();

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).unwrap();

        // Check if the compressed file exists
        assert!(output_path.exists());

        // uncompress the file
        let mut decoder = zstd::stream::Decoder::new(File::open(&output_path).unwrap()).unwrap();
        let mut writer = File::create(&result_path).unwrap();
        io::copy(&mut decoder, &mut writer).unwrap();   
        decoder.finish();

        // Check if the uncompressed file exists
        assert!(result_path.exists());

    }

    #[test]
    fn test_restore_large_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let result_path = dir.path().join("result.txt");
        let output_path = dir.path().join("output.zst");

        // Create a large test file
        let mut file = File::create(&input_path).unwrap();
        let buffer = vec![0; 1024 * 1024 * 10]; // 10MB
        file.write_all(&buffer).unwrap();

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).unwrap();

        // Restore the file
        restore_file(&output_path, &result_path).unwrap();

        // Check if the restored file exists
        assert!(result_path.exists());

    }
}
