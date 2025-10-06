use rtimelogger::config::migrate::run_fs_migration_with;
use serde_yaml::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir() -> PathBuf {
    let mut p = env::temp_dir();
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("rtimigrate_{}", suffix));
    // ensure clean
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create temp base");
    p
}
/// Helper: scrive un config YAML con la riga database.
fn write_config(path: &PathBuf, db_name: &str) {
    let content = format!("database: {}\n", db_name);
    let mut file = fs::File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

#[test]
fn test_config_dir_and_db_renamed_and_migration_logged() {
    // 1. crea un ambiente temporaneo
    let tmp = unique_temp_dir();
    let old_dir = tmp.join("rtimelog");
    let new_dir = tmp.join("rtimelogger");

    fs::create_dir_all(&old_dir).unwrap();

    // 2. crea file di config vecchio
    let old_conf = old_dir.join("rtimelog.conf");
    write_config(&old_conf, "rtimelog.sqlite");

    // 3. crea anche un db file dummy
    let old_db = old_dir.join("rtimelog.sqlite");
    fs::File::create(&old_db).unwrap();

    assert!(old_conf.exists());
    assert!(old_db.exists());

    // 4. esegui la migrazione simulata
    run_fs_migration_with(new_dir.clone(), old_dir.clone()).unwrap();

    // 5. verifica che la nuova dir e file ci siano
    assert!(new_dir.exists(), "new config dir should exist");
    let new_conf = new_dir.join("rtimelogger.conf");
    assert!(new_conf.exists(), "new config file should exist");

    // 6. verifica che il file db sia stato rinominato
    let new_db = new_dir.join("rtimelogger.sqlite");
    assert!(new_db.exists(), "new db file should exist");

    // 7. verifica che il config YAML punti al nuovo db
    let content = fs::read_to_string(&new_conf).unwrap();
    let yaml: Value = serde_yaml::from_str(&content).unwrap();
    let db = yaml.get("database").unwrap().as_str().unwrap();
    assert!(
        db.ends_with("rtimelogger.sqlite"),
        "config database should point to rtimelogger.sqlite"
    );
}
