use std::{fs, io};
use std::io::Write;
use std::path::Path;
use duckdb::Connection;
use crate::args::InitCommand;
use crate::config::Config;
use crate::CURRENT_DIR;
use crate::errors::BucketError;
use crate::utils::checks;
use log::debug;

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
    let repo_buckets_path = repo_path.join(".buckets");

    fs::create_dir_all(&repo_buckets_path)?;
    create_config_file(&repo_buckets_path)?;
    create_database(&repo_buckets_path)?;

    Ok(())
}

pub fn create_config_file(location: &Path) -> Result<(), BucketError> {
    // Define the default configuration
    let config = Config {
        ntp_server: "pool.ntp.org".to_string(),
        ip_check: "8.8.8.8".to_string(),
        url_check: "api.ipify.org".to_string(),
    };

    // Serialize the configuration to TOML format
    let toml_content = toml::to_string(&config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to serialize config: {}", e)))?;

    // Create the .buckets directory if it doesn't exist
    fs::create_dir_all(&location)?;

    // Write the configuration file
    let config_path = location.join("config");
    let mut file = fs::File::create(&config_path)?;
    file.write_all(toml_content.as_bytes())?;

    Ok(())
}

fn create_database(location: &Path) -> Result<(), BucketError> {
    let db_path = location.join("buckets.db");
    let connection = Connection::open(db_path)?;

    // The schema.sql file must be in the same directory as this source file
    let schema = include_str!("../sql/schema.sql");
    
    connection.execute_batch(schema)?;

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
