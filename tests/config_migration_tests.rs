use assert_cmd::Command;
use rusqlite::{Connection, OptionalExtension};
use std::env;
use std::fs;
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

#[test]
fn test_config_dir_and_db_renamed_and_migration_logged() {
    // Prepare unique temp base and set APPDATA for child process only
    let temp_base = unique_temp_dir();

    // Create old config dir: %APPDATA%/rtimelog
    let old_dir = temp_base.join("rtimelog");
    fs::create_dir_all(&old_dir).expect("create old config dir");

    // Create rtimelog.conf inside old dir referencing relative rtimelog.sqlite
    let old_conf = old_dir.join("rtimelog.conf");
    let yaml = "database: \"rtimelog.sqlite\"\ndefault_position: \"O\"\n";
    fs::write(&old_conf, yaml).expect("write old config");

    // Do not create the rtimelog.sqlite file — migration should update YAML and not fail

    // Prepare a separate DB for running migrations (so we don't lock files inside config dir)
    let mut db_path = env::temp_dir();
    db_path.push(format!(
        "rtimigrate_db_{}.sqlite",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db_path_str = db_path.to_string_lossy().to_string();
    // remove if exists
    let _ = fs::remove_file(&db_path);

    // Run the binary in a child process with APPDATA set to temp_base so Config picks it up
    Command::cargo_bin("rtimelogger")
        .expect("binary exists")
        .env("APPDATA", &temp_base)
        .args(["--db", &db_path_str, "--test", "init"])
        .assert()
        .success();

    // Verify new config dir exists: %APPDATA%/rtimelogger
    let new_dir = temp_base.join("rtimelogger");
    assert!(new_dir.exists(), "new config dir should exist");

    // Verify config file moved/created: rtimelogger.conf
    let new_conf = new_dir.join("rtimelogger.conf");
    assert!(new_conf.exists(), "new config file should exist");

    let content = fs::read_to_string(&new_conf).expect("read new config");
    assert!(
        content.contains("rtimelogger.sqlite"),
        "config should reference rtimelogger.sqlite"
    );

    // Verify DB log contains migration_applied marker for our version
    let conn = Connection::open(&db_path_str).expect("open migration DB");
    let mut stmt = conn
        .prepare(
            "SELECT target FROM log WHERE operation = 'migration_applied' ORDER BY id DESC LIMIT 1",
        )
        .expect("prepare stmt");
    let res: Option<String> = stmt
        .query_row([], |row| row.get(0))
        .optional()
        .expect("query row");
    assert!(
        res.is_some(),
        "migration_applied marker should be present in log"
    );
    let ver = res.unwrap();
    assert_eq!(ver, "20251006_0010_rename_rtimelog_to_rtimelogger");

    // Cleanup
    let _ = fs::remove_file(&db_path);
    let _ = fs::remove_dir_all(&temp_base);
}
