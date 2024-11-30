use log::debug;
use crate::args::InitCommand;
use crate::CURRENT_DIR;
use crate::errors::BucketError;
use crate::utils::checks;

pub fn execute(init_command : &InitCommand) -> Result<(), BucketError> {
    debug!("Initializing bucket repository {}", init_command.repo_name);

    match checks(init_command.repo_name.as_str()) {
        Err(e) => Err(e),
        Ok(_) =>
    }

    println!("init command");
    Ok(())
}

#[allow(dead_code)]
fn checks(repo_name: &str) -> Result<(), BucketError> {
    let repo_path = CURRENT_DIR.with(|dir| dir.join(repo_name));

    // Check if it already exists
    if repo_path.exists() {
        if repo_path.is_dir() {
            // Check if it is a valid bucket repo
            if checks::is_valid_bucket_repo(repo_path.as_path()) {
                return Err(BucketError::IoError(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "Bucket repository with same name already exists",
                )));
            } else {
                return Err(BucketError::IoError(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "Directory already exists",
                )));
            }
        } else {
            return Err(BucketError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File with the same name already exists",
            )));
        }
    }
    Ok(())
}
