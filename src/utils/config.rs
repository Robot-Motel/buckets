use crate::utils::checks::find_directory_in_parents;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RepositoryConfig {
    pub ntp_server: String,
    pub ip_check: String,
    pub url_check: String,
    pub external_database: Option<String>,
}

impl RepositoryConfig {
    pub(crate) fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let buckets_repo_path = find_directory_in_parents(&path, ".buckets").ok_or(
            std::io::Error::new(std::io::ErrorKind::NotFound, "No .buckets directory found"),
        )?;

        let mut file = File::open(buckets_repo_path.join("config"))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;
        let mut toml_string = String::new();
        file.read_to_string(&mut toml_string)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        toml::from_str(&toml_string)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        RepositoryConfig {
            ntp_server: "pool.ntp.org".to_string(),
            ip_check: "8.8.8.8".to_string(),
            url_check: "api.ipify.org".to_string(),
            external_database: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::BucketCommand;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_from_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let buckets_dir = temp_dir.path().join(".buckets");
        fs::create_dir(&buckets_dir).expect("Failed to create .buckets directory");

        // Create and write to the file
        let init_cmd = crate::commands::init::Init::new(&crate::args::InitCommand {
            shared: crate::args::SharedArguments::default(),
            repo_name: "test".to_string(),
            database: "postgresql".to_string(),
            external_database: Some("postgres://user:password@localhost/db".to_string()),
        });
        init_cmd
            .create_config_file(&buckets_dir.as_path())
            .expect("Failed to create config file");

        // Read the file
        let config = RepositoryConfig::from_file(temp_dir.path().to_path_buf())
            .expect("Failed to read config file");

        // Assertions
        assert_eq!(config.ip_check, "8.8.8.8");
        assert_eq!(config.ntp_server, "pool.ntp.org");
        assert_eq!(config.url_check, "api.ipify.org");
        assert_eq!(
            config.external_database,
            Some("postgres://user:password@localhost/db".to_string())
        );
    }

    #[test]
    fn test_config_default_values() {
        let config = RepositoryConfig::default();
        assert_eq!(config.ntp_server, "pool.ntp.org");
        assert_eq!(config.ip_check, "8.8.8.8");
        assert_eq!(config.url_check, "api.ipify.org");
        assert_eq!(config.external_database, None);
    }

    #[test]
    fn test_config_serialization() {
        let config = RepositoryConfig::default();
        let serialized = toml::to_string(&config).expect("Failed to serialize config");

        assert!(serialized.contains("ntp_server"));
        assert!(serialized.contains("ip_check"));
        assert!(serialized.contains("url_check"));
        assert!(serialized.contains("pool.ntp.org"));
        assert!(serialized.contains("8.8.8.8"));
        assert!(serialized.contains("api.ipify.org"));
        assert!(!serialized.contains("external_database"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_content = r#"
ntp_server = "custom.ntp.server"
ip_check = "1.1.1.1"
url_check = "custom.check.url"
external_database = "postgres://user:password@localhost/db"
"#;
        let config: RepositoryConfig =
            toml::from_str(toml_content).expect("Failed to deserialize config");

        assert_eq!(config.ntp_server, "custom.ntp.server");
        assert_eq!(config.ip_check, "1.1.1.1");
        assert_eq!(config.url_check, "custom.check.url");
        assert_eq!(
            config.external_database,
            Some("postgres://user:password@localhost/db".to_string())
        );
    }

    #[test]
    fn test_from_file_no_buckets_directory() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let result = RepositoryConfig::from_file(temp_dir.path().to_path_buf());

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        assert!(error.to_string().contains(".buckets"));
    }

    #[test]
    fn test_from_file_no_config_file() -> std::io::Result<()> {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let buckets_dir = temp_dir.path().join(".buckets");
        fs::create_dir(&buckets_dir)?;

        let result = RepositoryConfig::from_file(temp_dir.path().to_path_buf());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        Ok(())
    }

    #[test]
    fn test_from_file_corrupted_config() -> std::io::Result<()> {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let buckets_dir = temp_dir.path().join(".buckets");
        fs::create_dir(&buckets_dir)?;

        // Write invalid TOML content
        let config_path = buckets_dir.join("config");
        fs::write(&config_path, "invalid toml content { [ ] }")?;

        let result = RepositoryConfig::from_file(temp_dir.path().to_path_buf());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        Ok(())
    }

    #[test]
    fn test_from_file_nested_directory() -> std::io::Result<()> {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let buckets_dir = temp_dir.path().join(".buckets");
        fs::create_dir(&buckets_dir)?;

        // Create the config file
        let init_cmd = crate::commands::init::Init::new(&crate::args::InitCommand {
            shared: crate::args::SharedArguments::default(),
            repo_name: "test".to_string(),
            database: "postgresql".to_string(),
            external_database: None,
        });
        init_cmd
            .create_config_file(&buckets_dir.as_path())
            .expect("Failed to create config file");

        // Create nested directory and test from there
        let nested_dir = temp_dir.path().join("nested").join("directory");
        fs::create_dir_all(&nested_dir)?;

        let config = RepositoryConfig::from_file(nested_dir)?;
        assert_eq!(config.ip_check, "8.8.8.8");
        assert_eq!(config.ntp_server, "pool.ntp.org");
        assert_eq!(config.url_check, "api.ipify.org");
        assert_eq!(config.external_database, None);
        Ok(())
    }

    #[test]
    fn test_config_debug_format() {
        let config = RepositoryConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("RepositoryConfig"));
        assert!(debug_str.contains("ntp_server"));
        assert!(debug_str.contains("ip_check"));
        assert!(debug_str.contains("url_check"));
        assert!(debug_str.contains("external_database"));
    }
}
