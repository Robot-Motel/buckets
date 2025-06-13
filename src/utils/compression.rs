use std::{fs::File, io::{self, BufReader, BufWriter}, path::PathBuf};

use zstd::{stream::{copy_decode, copy_encode}, Decoder, Encoder};

pub fn compress_and_store_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    compression_level: i32,
) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut output_file = File::create(output_path)?;

    copy_encode( &input_file, &mut output_file, compression_level)?;

    Ok(())
}

pub fn restore_file(input_path: &PathBuf, output_path: &PathBuf) -> io::Result<()>{

    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    let mut decoder = Decoder::new(reader)?;
    copy_decode(&mut decoder, &mut writer)?;
    Ok(())

}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compress_and_store_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.zst");

        // Create a test file
        fs::write(&input_path, "Hello, world!").unwrap();

        // Compress the file
        compress_and_store_file(&input_path, &output_path, 3).unwrap();

        // Check if the compressed file exists
        assert!(output_path.exists());
    }

    #[test]
    fn test_restore_file() {
        let dir = tempdir().unwrap();
        let bucket_path = dir.path().to_path_buf();
        let compressed_file_path = bucket_path.join("input.zst");
        let restored_file_path = bucket_path.join("output.txt");

        // Create a test file
        fs::write(&restored_file_path, "Hello, world!").unwrap();
        compress_and_store_file(&restored_file_path, &compressed_file_path, 3).unwrap();
        fs::remove_file(&restored_file_path).unwrap();

        // Restore the file
        restore_file( &compressed_file_path, &restored_file_path).map_err(|e| {
            eprintln!("Failed to restore file: {}", e);
            e
        }).unwrap();

        // Check if the restored file exists and contains the expected content
        assert!(restored_file_path.exists());
        let content = fs::read_to_string(&restored_file_path).unwrap();
        assert_eq!(content, "Hello, world!");
    }
}
