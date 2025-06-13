use std::{fs::File, io::{self, BufReader, BufWriter}, path::{Path, PathBuf}};

use zstd::{stream::copy_decode, Decoder, Encoder};

use crate::data::commit::CommittedFile;

pub fn compress_and_store_file(
    input_path: &str,
    output_path: &Path,
    compression_level: i32,
) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    let mut reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);
    let mut encoder: Encoder<'static, BufWriter<File>> = Encoder::new(writer, compression_level)?;

    std::io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?; // Finalize the compression

    Ok(())
}

pub fn restore_file(bucket_path: &PathBuf, p1: &CommittedFile) -> io::Result<()>{
    let input_path = bucket_path.join(".b").join("storage").join(&p1.previous_hash.to_string());
    let output_path = bucket_path.join(&p1.name);

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
    use crate::data::commit::CommitStatus;

    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[test]
    fn test_compress_and_store_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.zst");

        // Create a test file
        fs::write(&input_path, "Hello, world!").unwrap();

        // Compress the file
        compress_and_store_file(input_path.to_str().unwrap(), &output_path, 3).unwrap();

        // Check if the compressed file exists
        assert!(output_path.exists());
    }

    #[test]
    fn test_restore_file() {
        let dir = tempdir().unwrap();
        let bucket_path = dir.path().to_path_buf();
        let storage_path = bucket_path.join(".b").join("storage");
        fs::create_dir_all(&storage_path).unwrap();

        let input_path = storage_path.join("input.zst");
        let output_path = bucket_path.join("output.txt");

        // Create a test file
        fs::write(&input_path, "Hello, world!").unwrap();

        // Create a CommittedFile instance
        let committed_file = CommittedFile {
            name: "output.txt".to_string(),
            previous_hash: blake3::hash("input".as_bytes()).into(),
            id: Uuid::new_v4(),
            hash: blake3::hash("some_hash".as_bytes()).into(),
            status: CommitStatus::Committed,
        };

        // Restore the file
        restore_file(&bucket_path, &committed_file).unwrap();

        // Check if the restored file exists and contains the expected content
        assert!(output_path.exists());
        let content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "Hello, world!");
    }
}
