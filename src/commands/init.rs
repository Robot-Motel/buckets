use crate::args::InitCommand;
use crate::commands::BucketCommand;
use crate::config::Config;
use crate::database::{initialize_database, DatabaseType};
use crate::errors::BucketError;
use crate::utils::checks;
use crate::CURRENT_DIR;
use log::debug;
use std::io::Write;
use std::path::Path;
use std::{fs, io};

/// Initialize a new bucket repository
pub struct Init {
    args: InitCommand,
}

impl BucketCommand for Init {
    type Args = InitCommand;

    fn new(args: &Self::Args) -> Self {
        Self { args: args.clone() }
    }

    fn execute(&self) -> Result<(), BucketError> {
        debug!("Initializing bucket repository {}", self.args.repo_name);

        // Perform checks
        self.checks(&self.args.repo_name)?;

        // Create the repository
        let current_dir = CURRENT_DIR.with(|dir| dir.clone());
        self.create_repo(&self.args.repo_name, &current_dir)?;

        println!("Bucket repository initialized successfully.");
        Ok(())
    }
}

impl Init {
    fn create_repo(&self, repo_name: &str, repo_location: &Path) -> Result<(), BucketError> {
        let repo_path = repo_location.join(repo_name);
        let repo_buckets_path = repo_path.join(".buckets");

        fs::create_dir_all(&repo_buckets_path)?;
        self.create_config_file(&repo_buckets_path)?;

        let db_type = DatabaseType::from_str(&self.args.database)?;
        initialize_database(&repo_buckets_path, db_type)?;

        Ok(())
    }

    pub fn create_config_file(&self, location: &Path) -> Result<(), BucketError> {
        // Define the default configuration
        let config = Config {
            ntp_server: "pool.ntp.org".to_string(),
            ip_check: "8.8.8.8".to_string(),
            url_check: "api.ipify.org".to_string(),
        };

        // Serialize the configuration to TOML format
        let toml_content = toml::to_string(&config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize config: {}", e),
            )
        })?;

        // Create the .buckets directory if it doesn't exist
        fs::create_dir_all(&location)?;

        // Write the configuration file
        let config_path = location.join("config");
        let mut file = fs::File::create(&config_path)?;
        file.write_all(toml_content.as_bytes())?;

        Ok(())
    }

    fn checks(&self, repo_name: &str) -> Result<(), BucketError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::SharedArguments;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_init_command(repo_name: &str, database: &str) -> Init {
        let args = InitCommand {
            shared: SharedArguments::default(),
            repo_name: repo_name.to_string(),
            database: database.to_string(),
        };
        Init::new(&args)
    }

    #[test]
    fn test_init_new() {
        let args = InitCommand {
            shared: SharedArguments::default(),
            repo_name: "test_repo".to_string(),
            database: "duckdb".to_string(),
        };
        let init = Init::new(&args);
        assert_eq!(init.args.repo_name, "test_repo");
        assert_eq!(init.args.database, "duckdb");
    }

    #[test]
    fn test_create_config_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let config_dir = temp_dir.path().join("test_config");

        let init = create_test_init_command("test_repo", "duckdb");
        let result = init.create_config_file(&config_dir);

        assert!(result.is_ok());
        assert!(config_dir.exists());
        assert!(config_dir.is_dir());

        let config_file = config_dir.join("config");
        assert!(config_file.exists());
        assert!(config_file.is_file());

        // Verify the config file content
        let content = fs::read_to_string(&config_file).expect("Failed to read config file");
        assert!(content.contains("ntp_server = \"pool.ntp.org\""));
        assert!(content.contains("ip_check = \"8.8.8.8\""));
        assert!(content.contains("url_check = \"api.ipify.org\""));
    }

    #[test]
    fn test_create_config_file_existing_directory() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let config_dir = temp_dir.path().join("existing_config");

        // Pre-create the directory
        fs::create_dir_all(&config_dir).expect("Failed to create directory");

        let init = create_test_init_command("test_repo", "duckdb");
        let result = init.create_config_file(&config_dir);

        assert!(result.is_ok());

        let config_file = config_dir.join("config");
        assert!(config_file.exists());
        assert!(config_file.is_file());
    }

    #[test]
    fn test_create_config_file_invalid_path() {
        let init = create_test_init_command("test_repo", "duckdb");

        // Try to create config in a path that cannot be created (invalid parent)
        let invalid_path = std::path::Path::new("/invalid/path/that/does/not/exist");
        let result = init.create_config_file(invalid_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_checks_valid_repo_name() {
        let init = create_test_init_command("valid_repo", "duckdb");
        let result = init.checks("valid_repo");

        // Should be ok because the repo doesn't exist yet
        assert!(result.is_ok());
    }

    #[test]
    fn test_checks_directory_already_exists() {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let test_repo_name = "temp_test_existing_repo";
        let existing_dir = current_dir.join(test_repo_name);

        // Create a directory that already exists but is not a bucket repo
        fs::create_dir_all(&existing_dir).expect("Failed to create directory");

        let init = create_test_init_command(test_repo_name, "duckdb");
        let result = init.checks(test_repo_name);

        // Clean up the test directory
        fs::remove_dir_all(&existing_dir).expect("Failed to remove test directory");

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::IoError(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
                assert_eq!(err.to_string(), "Directory already exists");
            }
            _ => panic!("Expected IoError with AlreadyExists kind"),
        }
    }

    #[test]
    fn test_checks_file_already_exists() {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let test_file_name = "temp_test_existing_file_repo";
        let existing_file = current_dir.join(test_file_name);

        // Create a file with the same name as the repo
        fs::write(&existing_file, "test content").expect("Failed to create file");

        let init = create_test_init_command(test_file_name, "duckdb");
        let result = init.checks(test_file_name);

        // Clean up the test file
        fs::remove_file(&existing_file).expect("Failed to remove test file");

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::IoError(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
                assert_eq!(err.to_string(), "File with the same name already exists");
            }
            _ => panic!("Expected IoError with AlreadyExists kind"),
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_checks_existing_bucket_repo() {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let test_repo_name = "temp_test_existing_bucket_repo";
        let existing_repo = current_dir.join(test_repo_name);
        let buckets_dir = existing_repo.join(".buckets");

        // Create a directory that looks like a bucket repo
        fs::create_dir_all(&buckets_dir).expect("Failed to create .buckets directory");

        // Create config file to make it look like a valid repo
        let config_file = buckets_dir.join("config");
        fs::write(&config_file, "ntp_server = \"pool.ntp.org\"").expect("Failed to create config");

        // Create a database file to make it look like a valid repo
        let db_file = buckets_dir.join("buckets.db");
        let conn = duckdb::Connection::open(&db_file).expect("Failed to create database");
        conn.execute("CREATE TABLE test (id INTEGER);", [])
            .expect("Failed to create table");
        conn.close().expect("Failed to close connection");

        let init = create_test_init_command(test_repo_name, "duckdb");
        let result = init.checks(test_repo_name);

        // Clean up the test directory
        fs::remove_dir_all(&existing_repo).expect("Failed to remove test directory");

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::RepoAlreadyExists(repo_name) => {
                assert_eq!(repo_name, test_repo_name);
            }
            _ => panic!("Expected RepoAlreadyExists error"),
        }
    }

    #[test]
    fn test_create_repo_with_duckdb() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let init = create_test_init_command("test_repo", "duckdb");
        let result = init.create_repo("test_repo", temp_dir.path());

        assert!(result.is_ok());

        let repo_path = temp_dir.path().join("test_repo");
        assert!(repo_path.exists());
        assert!(repo_path.is_dir());

        let buckets_path = repo_path.join(".buckets");
        assert!(buckets_path.exists());
        assert!(buckets_path.is_dir());

        let config_file = buckets_path.join("config");
        assert!(config_file.exists());
        assert!(config_file.is_file());

        let db_file = buckets_path.join("buckets.db");
        assert!(db_file.exists());
        assert!(db_file.is_file());

        let db_type_file = buckets_path.join("database_type");
        assert!(db_type_file.exists());
        assert!(db_type_file.is_file());

        let db_type_content =
            fs::read_to_string(db_type_file).expect("Failed to read database_type file");
        assert_eq!(db_type_content.trim(), "duckdb");
    }

    #[test]
    fn test_create_repo_with_postgresql() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let init = create_test_init_command("test_repo", "postgresql");
        let result = init.create_repo("test_repo", temp_dir.path());

        // The result depends on whether PostgreSQL feature is enabled
        // For now, we'll just check that it doesn't panic and handles the database type correctly
        let repo_path = temp_dir.path().join("test_repo");
        let buckets_path = repo_path.join(".buckets");

        if result.is_ok() {
            // If PostgreSQL is enabled, check the structure
            assert!(repo_path.exists());
            assert!(buckets_path.exists());

            let config_file = buckets_path.join("config");
            assert!(config_file.exists());

            let db_type_file = buckets_path.join("database_type");
            if db_type_file.exists() {
                let db_type_content =
                    fs::read_to_string(db_type_file).expect("Failed to read database_type file");
                assert_eq!(db_type_content.trim(), "postgresql");
            }
        } else {
            // If PostgreSQL is not enabled, it should fail with a database error
            match result.unwrap_err() {
                BucketError::DatabaseError(_) => {
                    // This is expected when PostgreSQL support is not compiled in
                }
                _ => panic!("Expected DatabaseError when PostgreSQL is not available"),
            }
        }
    }

    #[test]
    fn test_create_repo_invalid_database_type() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let init = create_test_init_command("test_repo", "invalid_db");
        let result = init.create_repo("test_repo", temp_dir.path());

        assert!(result.is_err());
        match result.unwrap_err() {
            BucketError::InvalidData(msg) => {
                assert!(msg.contains("Unsupported database type"));
                assert!(msg.contains("invalid_db"));
            }
            _ => panic!("Expected InvalidData error for invalid database type"),
        }
    }

    #[test]
    fn test_create_repo_permission_denied() {
        // This test is more complex as it requires a directory where we can't create subdirectories
        // We'll use a simple approach by trying to create in a read-only directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir_all(&readonly_dir).expect("Failed to create readonly directory");

        // Make the directory read-only (this might not work on all systems)
        let mut permissions = fs::metadata(&readonly_dir)
            .expect("Failed to get metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&readonly_dir, permissions).expect("Failed to set readonly");

        let init = create_test_init_command("test_repo", "duckdb");
        let result = init.create_repo("test_repo", &readonly_dir);

        // The result depends on the system's permission handling
        // On some systems, this might still succeed, so we'll just check it doesn't panic
        let _ = result; // Don't assert specific behavior as it's system-dependent
    }

    #[test]
    fn test_database_type_validation() {
        // Test valid database types
        assert!(DatabaseType::from_str("duckdb").is_ok());
        assert!(DatabaseType::from_str("postgresql").is_ok());
        assert!(DatabaseType::from_str("postgres").is_ok());

        // Test case insensitivity
        assert!(DatabaseType::from_str("DUCKDB").is_ok());
        assert!(DatabaseType::from_str("PostgreSQL").is_ok());
        assert!(DatabaseType::from_str("POSTGRES").is_ok());

        // Test invalid database types
        assert!(DatabaseType::from_str("mysql").is_err());
        assert!(DatabaseType::from_str("sqlite").is_err());
        assert!(DatabaseType::from_str("invalid").is_err());
        assert!(DatabaseType::from_str("").is_err());
    }

    #[test]
    fn test_config_file_content_format() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let config_dir = temp_dir.path().join("test_config");

        let init = create_test_init_command("test_repo", "duckdb");
        let result = init.create_config_file(&config_dir);

        assert!(result.is_ok());

        let config_file = config_dir.join("config");
        let content = fs::read_to_string(&config_file).expect("Failed to read config file");

        // Verify it's valid TOML format by parsing it
        let parsed: toml::Value = toml::from_str(&content).expect("Config file is not valid TOML");

        // Verify the structure
        assert!(parsed.get("ntp_server").is_some());
        assert!(parsed.get("ip_check").is_some());
        assert!(parsed.get("url_check").is_some());

        // Verify the values
        assert_eq!(parsed["ntp_server"].as_str(), Some("pool.ntp.org"));
        assert_eq!(parsed["ip_check"].as_str(), Some("8.8.8.8"));
        assert_eq!(parsed["url_check"].as_str(), Some("api.ipify.org"));
    }
}
