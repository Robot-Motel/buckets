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

fn create_database(location: &Path) -> Result<(), duckdb::Error> {
    let db_path = location.join("buckets.db");

    // Not using the connection from the struct because it is in-memory and immutable
    let connection = Connection::open(db_path)?;

    match connection.execute(
        "CREATE TABLE buckets (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL
        )",
        [],
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("Error creating 'buckets' table: {}", e);
            return Err(e);
        }
    }

    match connection.execute(
        "CREATE TABLE commits (
            id UUID PRIMARY KEY,
            bucket_id UUID NOT NULL,
            message TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (bucket_id) REFERENCES buckets (id)
        )",
        [],
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("Error creating 'commits' table: {}", e);
            return Err(e);
        }
    }

    match connection.execute(
        "CREATE TABLE files (
            id UUID PRIMARY KEY,
            commit_id UUID NOT NULL,
            file_path TEXT NOT NULL,
            hash TEXT NOT NULL,
            FOREIGN KEY (commit_id) REFERENCES commits (id),
            UNIQUE (commit_id, file_path, hash)
        )",
        [],
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("Error creating 'files' table: {}", e);
            return Err(e);
        }
    }

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
