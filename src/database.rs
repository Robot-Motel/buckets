use crate::errors::BucketError;
use std::path::Path;
use std::{env, fs};

#[cfg(feature = "postgres")]
use postgres;
#[cfg(feature = "postgres")]
use postgresql_embedded;

#[derive(Debug, Clone, Copy)]
pub enum DatabaseType {
    DuckDB,
    PostgreSQL,
}

impl DatabaseType {
    pub fn from_str(s: &str) -> Result<Self, BucketError> {
        match s.to_lowercase().as_str() {
            "duckdb" => Ok(DatabaseType::DuckDB),
            "postgresql" | "postgres" => Ok(DatabaseType::PostgreSQL),
            _ => Err(BucketError::InvalidData(format!(
                "Unsupported database type: {}",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::DuckDB => "duckdb",
            DatabaseType::PostgreSQL => "postgresql",
        }
    }
}

#[allow(dead_code)]
pub fn get_database_type() -> Result<DatabaseType, BucketError> {
    let current_dir = env::current_dir()?;
    let buckets_dir = crate::utils::utils::find_directory_in_parents(&current_dir, ".buckets")
        .ok_or(BucketError::NotInRepo)?;

    let db_type_file = buckets_dir.join("database_type");
    if db_type_file.exists() {
        let content = fs::read_to_string(db_type_file)?;
        DatabaseType::from_str(content.trim())
    } else {
        // Default to DuckDB for backward compatibility
        Ok(DatabaseType::DuckDB)
    }
}

#[allow(dead_code)]
pub fn get_database_path() -> Result<std::path::PathBuf, BucketError> {
    let current_dir = env::current_dir()?;
    let buckets_dir = crate::utils::utils::find_directory_in_parents(&current_dir, ".buckets")
        .ok_or(BucketError::NotInRepo)?;

    let db_type = get_database_type()?;
    match db_type {
        DatabaseType::DuckDB => Ok(buckets_dir.join("buckets.db")),
        DatabaseType::PostgreSQL => Ok(buckets_dir.join("postgres_data")),
    }
}

pub fn create_duckdb_connection(path: &Path) -> Result<duckdb::Connection, BucketError> {
    duckdb::Connection::open(path).map_err(BucketError::DuckDB)
}

#[cfg(feature = "postgres")]
pub fn create_postgres_connection_and_execute_schema(
    data_dir: &Path,
    schema: &str,
) -> Result<(), BucketError> {
    use postgresql_embedded::{PostgreSQL, Settings};

    let settings = Settings {
        data_dir: data_dir.to_path_buf(),
        ..Default::default()
    };

    let mut server = PostgreSQL::new(settings);

    // Use a simple runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BucketError::DatabaseError(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        server.setup().await.map_err(|e| {
            BucketError::DatabaseError(format!("Failed to setup PostgreSQL: {}", e))
        })?;

        server.start().await.map_err(|e| {
            BucketError::DatabaseError(format!("Failed to start PostgreSQL: {}", e))
        })?;

        // Wait a bit for the server to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok::<(), BucketError>(())
    })?;

    let url = server.settings().url("postgres");

    // Connect and execute schema while server is alive
    let mut client = postgres::Client::connect(&url, postgres::NoTls).map_err(|e| {
        BucketError::DatabaseError(format!("Failed to connect to PostgreSQL: {}", e))
    })?;

    client
        .batch_execute(schema)
        .map_err(|e| BucketError::DatabaseError(format!("Failed to execute schema: {}", e)))?;

    Ok(())
}

#[cfg(not(feature = "postgres"))]
#[allow(dead_code)]
pub fn create_postgres_connection_and_execute_schema(
    _data_dir: &Path,
    _schema: &str,
) -> Result<(), BucketError> {
    Err(BucketError::DatabaseError(
        "PostgreSQL support not compiled in".to_string(),
    ))
}

pub fn initialize_database(location: &Path, db_type: DatabaseType) -> Result<(), BucketError> {
    let schema = include_str!("sql/schema.sql");

    match db_type {
        DatabaseType::DuckDB => {
            let db_path = location.join("buckets.db");
            let connection = create_duckdb_connection(&db_path)?;
            connection.execute_batch(schema)?;
        }
        DatabaseType::PostgreSQL => {
            #[cfg(feature = "postgres")]
            {
                let data_dir = location.join("postgres_data");
                fs::create_dir_all(&data_dir)?;
                create_postgres_connection_and_execute_schema(&data_dir, schema)?;
            }
            #[cfg(not(feature = "postgres"))]
            {
                return Err(BucketError::DatabaseError(
                    "PostgreSQL support not compiled in. Build with --features postgres to enable."
                        .to_string(),
                ));
            }
        }
    }

    // Write database type to config
    let config_path = location.join("database_type");
    fs::write(config_path, db_type.as_str())?;

    Ok(())
}
