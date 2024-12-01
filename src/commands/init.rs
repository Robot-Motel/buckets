use std::path::Path;
use log::debug;
use crate::args::InitCommand;
use crate::CURRENT_DIR;
use crate::errors::BucketError;
use crate::utils::checks;

pub fn execute(init_command: &InitCommand) -> Result<(), BucketError> {
    debug!("Initializing bucket repository {}", init_command.repo_name);

    // Perform checks
    checks(&init_command.repo_name)?;

    // Create the repository
    let current_dir = CURRENT_DIR.with(|dir| dir.clone());
    create_repo(&init_command.repo_name, &current_dir)?;

    println!("Bucket repository initialized successfully.");
    Ok(())
}

fn create_repo(repo_name: &str, repo_location: &Path) -> Result<(), BucketError> {
    let repo_path = repo_location.join(repo_name);

    std::fs::create_dir_all(repo_path.join(".buckets"))?;
    create_config_file(&repo_path)?;
    create_database(&repo_path)?;

    Ok(())
}

fn create_config_file(repo_path: &Path) -> Result<(), BucketError> {
    log::info!("TODO: Create the config file with the required fields");
    let config_file_path = repo_path.join(".buckets").join("config");
    std::fs::File::create(config_file_path)?;
    Ok(())
}

fn create_database(repo_path: &Path) -> Result<(), BucketError> {
    log::info!("TODO: Create the database file with DuckDB crate");
    let database_file_path = repo_path.join(".buckets").join("buckets.db");
    std::fs::File::create(database_file_path)?;
    Ok(())
}

fn checks(repo_name: &str) -> Result<(), BucketError> {
    let repo_path = CURRENT_DIR.with(|dir| dir.join(repo_name));

    if repo_path.exists() {
        if repo_path.is_dir() {
            if checks::is_valid_bucket_repo(repo_path.as_path()) {
                return Err(BucketError::RepoAlreadyExists(repo_name.to_string()));
            }
            return Err(BucketError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Directory already exists",
            )));
        } else {
            return Err(BucketError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File with the same name already exists",
            )));
        }
    }
    Ok(())
}
