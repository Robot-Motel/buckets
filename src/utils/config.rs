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
}

impl RepositoryConfig {
    pub(crate) fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let buckets_repo_path = find_directory_in_parents(&path, ".buckets")
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No .buckets directory found",
            ))?;

        let mut file = File::open(buckets_repo_path.join("config"))
            .map_err(
                |e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()),
            )?;
        let mut toml_string = String::new();
        file.read_to_string(&mut toml_string)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        Ok(toml::from_str(&toml_string).expect("Failed to parse toml string"))
    }
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        RepositoryConfig {
            ntp_server: "pool.ntp.org".to_string(),
            ip_check: "8.8.8.8".to_string(),
            url_check: "api.ipify.org".to_string(),
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use crate::commands::BucketCommand;

    #[test]
    fn test_from_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let buckets_dir = temp_dir.path().join(".buckets");
        fs::create_dir(&buckets_dir).expect("Failed to create .buckets directory");

        // Create and write to the file
        let init_cmd = crate::commands::init::Init::new(&crate::args::InitCommand {
            shared: crate::args::SharedArguments::default(),
            repo_name: "test".to_string(),
        });
        init_cmd.create_config_file(&buckets_dir.as_path()).expect("Failed to create config file");

        // Read the file
        let config = RepositoryConfig::from_file(temp_dir.path().to_path_buf()).expect("Failed to read config file");

        // Assertions
        assert_eq!(config.ip_check, "8.8.8.8");
        assert_eq!(config.ntp_server, "pool.ntp.org");
        assert_eq!(config.url_check, "api.ipify.org");
    }
}
