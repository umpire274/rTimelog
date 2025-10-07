// filepath: x:\Development\workspace\RustProjects\rTimelog\src\config\migrate.rs
use rusqlite::{Connection, Error, OptionalExtension};
use serde_yaml::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Helper: returns true if the path's filename matches the legacy DB name (case-insensitive on Windows)
fn is_old_db_name(db_path: &Path) -> bool {
    if cfg!(target_os = "windows") {
        db_path
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase() == "rtimelog.sqlite")
            .unwrap_or(false)
    } else {
        db_path
            .file_name()
            .map(|s| s.to_string_lossy() == "rtimelog.sqlite")
            .unwrap_or(false)
    }
}

// Helper: preserve the directory contained in the original `dbstr` and replace only the file name
fn preserve_db_filename(dbstr: &str, new_db_name: &str) -> String {
    PathBuf::from(dbstr)
        .with_file_name(new_db_name)
        .to_string_lossy()
        .to_string()
}

// Helper: try to rename a file, fallback to copy+remove if rename fails
fn move_or_copy(from: &Path, to: &Path) -> io::Result<()> {
    // If source does not exist, nothing to do
    if !from.exists() {
        return Ok(());
    }
    // If target already exists, skip to avoid overwriting
    if to.exists() {
        return Ok(());
    }

    // Try to rename; on failure, fallback to copy+remove
    if fs::rename(from, to).is_err() {
        fs::copy(from, to)?;
        let _ = fs::remove_file(from);
    }
    Ok(())
}

// Helper: read YAML config, detect legacy DB filename and (if present) rename the DB file on disk
// and update the YAML 'database' value, preserving the original directory. Returns Ok(true) if
// an update was performed, Ok(false) if no change was needed.
fn update_db_reference_in_conf_io(new_conf: &Path, new_dir: &Path) -> io::Result<bool> {
    let content = fs::read_to_string(new_conf)?;
    if let Ok(mut yaml) = serde_yaml::from_str::<Value>(&content)
        && let Some(map) = yaml.as_mapping_mut()
    {
        let key = Value::String("database".to_string());
        if let Some(val) = map.get(&key)
            && let Some(dbstr) = val.as_str()
        {
            let db_path = PathBuf::from(dbstr);
            if is_old_db_name(&db_path) {
                let actual_old_db = if db_path.is_absolute() {
                    db_path.clone()
                } else {
                    new_dir.join(&db_path)
                };
                let actual_new_db = actual_old_db.with_file_name("rtimelogger.sqlite");

                if actual_old_db.exists() {
                    if actual_new_db.exists() {
                        return Err(io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            format!("Target DB already exists: {:?}", actual_new_db),
                        ));
                    }
                    if fs::rename(&actual_old_db, &actual_new_db).is_err() {
                        fs::copy(&actual_old_db, &actual_new_db)?;
                        let _ = fs::remove_file(&actual_old_db);
                    }
                }

                // update YAML: preserve directory, change filename only
                let new_db_str = preserve_db_filename(dbstr, "rtimelogger.sqlite");
                map.insert(key.clone(), Value::String(new_db_str));

                // write back
                let serialized = serde_yaml::to_string(&yaml).map_err(|e| {
                    io::Error::other(format!(
                        "Failed to serialize YAML for {:?}: {}",
                        new_conf, e
                    ))
                })?;
                fs::write(new_conf, serialized)?;

                return Ok(true);
            }
        }
    }
    Ok(false)
}

const VERSION: &str = "20251006_0010_rename_rtimelog_to_rtimelogger";

/// Run the config migration once. Idempotent when used via run_pending_migrations which
/// already checks applied versions. This function returns Err on critical failures so the
/// caller (migration runner) won't mark the migration as applied.
pub fn run_config_migration(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            operation TEXT NOT NULL,
            target TEXT DEFAULT '',
            message TEXT NOT NULL
        );",
    )?;

    let mut chk = conn.prepare(
        "SELECT 1 FROM log WHERE operation = 'migration_applied' AND target = ?1 LIMIT 1",
    )?;
    if chk.query_row([VERSION], |_| Ok(())).optional()?.is_some() {
        return Ok(());
    }

    let new_dir = super::Config::config_dir();
    let old_dir = old_config_dir();
    let mut actions: Vec<String> = Vec::new();

    // 1) rename directory if needed
    if old_dir.exists() && !new_dir.exists() {
        fs::rename(&old_dir, &new_dir).map_err(|e| {
            Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(format!(
                    "Failed to rename config dir {:?} -> {:?}: {}",
                    old_dir, new_dir, e
                )),
            )
        })?;
    }

    // 2) rename config file if needed (check in new_dir)
    let old_conf = new_dir.join("rtimelog.conf");
    let new_conf = new_dir.join("rtimelogger.conf");

    // attempt to move/copy file if present; helper will be a no-op if source missing or target exists
    move_or_copy(&old_conf, &new_conf).map_err(|e| {
        Error::SqliteFailure(
            rusqlite::ffi::Error::new(1),
            Some(format!(
                "Failed to move config file {:?} -> {:?}: {}",
                old_conf, new_conf, e
            )),
        )
    })?;

    // 3) update database name inside YAML
    if new_conf.exists() {
        match update_db_reference_in_conf_io(&new_conf, &new_dir) {
            Ok(updated) => {
                if updated {
                    actions.push("Updated config database reference".into());
                }
            }
            Err(e) => {
                return Err(Error::SqliteFailure(
                    rusqlite::ffi::Error::new(1),
                    Some(format!(
                        "Failed to update config database reference {:?}: {}",
                        new_conf, e
                    )),
                ));
            }
        }
    }

    if !actions.is_empty() {
        println!("ℹ️  Migration ({}): {}", VERSION, actions.join("; "));
    }

    Ok(())
}

pub fn run_fs_migration_with(new_dir: PathBuf, old_dir: PathBuf) -> io::Result<()> {
    // tutta la logica di run_fs_migration, ma usando i parametri
    // ----------------------------------------------------------
    if old_dir.exists() && !new_dir.exists() && fs::rename(&old_dir, &new_dir).is_err() {
        fs::create_dir_all(&new_dir)?;
        for ent in fs::read_dir(&old_dir)? {
            let ent = ent?;
            let from = ent.path();
            let fname = match from.file_name() {
                Some(n) => n,
                None => continue,
            };
            let to = new_dir.join(fname);
            if fs::rename(&from, &to).is_err() {
                fs::copy(&from, &to)?;
                let _ = fs::remove_file(&from);
            }
        }
        let _ = fs::remove_dir(&old_dir);
    }

    // rename config file if present
    let old_conf = new_dir.join("rtimelog.conf");
    let new_conf = new_dir.join("rtimelogger.conf");

    move_or_copy(&old_conf, &new_conf)?;

    // update db reference inside config
    if new_conf.exists() {
        let content = fs::read_to_string(&new_conf)?;
        if let Ok(mut yaml) = serde_yaml::from_str::<Value>(&content)
            && let Some(map) = yaml.as_mapping_mut()
        {
            let key = Value::String("database".to_string());
            if let Some(val) = map.get(&key)
                && let Some(dbstr) = val.as_str()
            {
                let db_path = PathBuf::from(dbstr);
                let is_old_db = db_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_lowercase() == "rtimelog.sqlite")
                    .unwrap_or(false);

                if is_old_db {
                    let actual_old_db = if db_path.is_absolute() {
                        db_path.clone()
                    } else {
                        new_dir.join(&db_path)
                    };
                    let actual_new_db = actual_old_db.with_file_name("rtimelogger.sqlite");

                    move_or_copy(&actual_old_db, &actual_new_db)?;

                    let new_db_str = PathBuf::from(dbstr)
                        .with_file_name("rtimelogger.sqlite")
                        .to_string_lossy()
                        .to_string();
                    map.insert(key.clone(), Value::String(new_db_str));
                    let serialized = serde_yaml::to_string(&yaml)
                        .map_err(|e| io::Error::other(format!("serialize error: {}", e)))?;
                    fs::write(&new_conf, serialized)
                        .map_err(|e| io::Error::other(format!("write error: {}", e)))?;
                }
            }
        }
    }

    Ok(())
}

/// Filesystem-only migration: rename old config dir/file and DB from 'rtimelog' to 'rtimelogger'.
/// This does NOT open or write to the database; it only manipulates files so it can run before
/// a DB connection exists. It returns io::Result so the caller can decide how to handle failures.
pub fn run_fs_migration() -> io::Result<()> {
    run_fs_migration_with(super::Config::config_dir(), old_config_dir())
}

fn old_config_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        Path::new(&appdata).join("rtimelog")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(".rtimelog")
    }
}

pub fn migrate_add_show_weekday(conn: &Connection) -> Result<(), Error> {
    let version = "20251008_0011_add_show_weekday";

    // verifica se già applicata
    let mut chk = conn.prepare(
        "SELECT 1 FROM log WHERE operation = 'migration_applied' AND target = ?1 LIMIT 1",
    )?;
    if chk.query_row([version], |_| Ok(())).optional()?.is_some() {
        return Ok(()); // già applicata
    }

    let conf_file = super::Config::config_file();
    if conf_file.exists() {
        let content = fs::read_to_string(&conf_file).map_err(|e| {
            Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(format!("Failed to read config {:?}: {}", conf_file, e)),
            )
        })?;
        if let Ok(mut yaml) = serde_yaml::from_str::<Value>(&content)
            && let Some(map) = yaml.as_mapping_mut()
        {
            let key = Value::String("show_weekday".to_string());
            if !map.contains_key(&key) {
                map.insert(key.clone(), Value::String("None".to_string()));

                // Serializza
                let serialized = serde_yaml::to_string(&yaml).map_err(|e| {
                    Error::SqliteFailure(
                        rusqlite::ffi::Error::new(1),
                        Some(format!("Failed to serialize config {:?}: {}", conf_file, e)),
                    )
                })?;

                // Aggiungi commento YAML subito dopo la riga 'show_weekday'
                let mut new_content = String::new();
                for line in serialized.lines() {
                    new_content.push_str(line);
                    new_content.push('\n');
                    if line.starts_with("show_weekday:") {
                        new_content.push_str(
                            "  # show-weekday parameter options:\n\
                                 #   None   → do not show weekday\n\
                                 #   Short  → Mo, Tu, We, Th, Fr, Sa, Su\n\
                                 #   Medium → Mon, Tue, Wed, Thu, Fri, Sat, Sun\n\
                                 #   Long   → Monday, Tuesday, ...\n",
                        );
                    }
                }

                fs::write(&conf_file, new_content).map_err(|e| {
                    Error::SqliteFailure(
                        rusqlite::ffi::Error::new(1),
                        Some(format!(
                            "Failed to write updated config {:?}: {}",
                            conf_file, e
                        )),
                    )
                })?;
            }
        }
    }

    println!(
        "✅ Migration applied: {} — Add show_weekday parameter to config",
        version
    );
    Ok(())
}
