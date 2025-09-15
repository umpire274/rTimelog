use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database: String,
    pub default_position: String,
}

impl Config {
    /// Return the standard configuration directory depending on the platform
    pub fn config_dir() -> PathBuf {
        if cfg!(target_os = "windows") {
            let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(appdata).join("rtimelog")
        } else {
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".rtimelog")
        }
    }

    /// Return the full path of the config file
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("rtimelog.conf")
    }

    /// Return the full path of the SQLite database
    pub fn database_file() -> PathBuf {
        Self::config_dir().join("rtimelog.sqlite")
    }

    /// Load configuration from file, or return defaults if not found
    pub fn load() -> Self {
        let path = Self::config_file();

        if path.exists() {
            let content = fs::read_to_string(&path).expect("❌ Failed to read configuration file");
            serde_yaml::from_str(&content).expect("❌ Failed to parse configuration file")
        } else {
            Config {
                database: Self::database_file().to_string_lossy().to_string(),
                default_position: "O".to_string(),
            }
        }
    }

    /// Initialize configuration and database files
    pub fn init_all(custom_name: Option<String>) -> io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        // DB name: user provided or default
        let db_path = if let Some(name) = custom_name {
            let p = std::path::Path::new(&name);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                dir.join(p)
            }
        } else {
            dir.join("rtimelog.sqlite")
        };

        let config = Config {
            database: db_path.to_string_lossy().to_string(),
            default_position: "O".to_string(),
        };

        // Write config file
        let yaml = serde_yaml::to_string(&config).unwrap();
        let mut file = fs::File::create(Self::config_file())?;
        file.write_all(yaml.as_bytes())?;

        // Create empty DB file if not exists
        if !db_path.exists() {
            fs::File::create(&db_path)?;
        }

        println!("✅ Config file: {:?}", Self::config_file());
        println!("✅ Database:    {:?}", db_path);

        Ok(())
    }
}
