use assert_cmd::Command;
use serde_json::Value;
use std::env;
use std::path::PathBuf;

/// Create a unique test DB path inside the system temp dir
fn setup_test_db(name: &str) -> String {
    let mut path: PathBuf = env::temp_dir();
    path.push(format!("{}_rtimelogger.sqlite", name));
    let db_path = path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&db_path);
    db_path
}

#[test]
fn test_create_missing_in_when_only_out_exists() {
    let db_path = setup_test_db("missing_event_in");

    // Init DB
    Command::cargo_bin("rtimelogger")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Create only an OUT event for the date (no start)
    Command::cargo_bin("rtimelogger")
        .unwrap()
        .args(["--db", &db_path, "add", "2025-10-01", "--out", "17:00"])
        .assert()
        .success();

    // Verify currently there is a single out event
    let out_only = Command::cargo_bin("rtimelogger")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events", "--json"])
        .output()
        .expect("failed to list events (out-only)");
    assert!(out_only.status.success());
    let json: Value = serde_json::from_slice(&out_only.stdout).expect("invalid JSON");
    let arr = json.as_array().expect("expected array");
    assert_eq!(arr.len(), 1, "Expected exactly 1 event (out only)");
    assert_eq!(arr[0]["kind"], "out");

    // Now use explicit edit on pair 1 to add the missing IN event
    Command::cargo_bin("rtimelogger")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-10-01",
            "--edit",
            "--pair",
            "1",
            "--in",
            "09:00",
        ])
        .assert()
        .success();

    // Re-list events and expect both in and out
    let both = Command::cargo_bin("rtimelogger")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events", "--json"])
        .output()
        .expect("failed to list events (after creating in)");
    assert!(both.status.success());
    let json2: Value = serde_json::from_slice(&both.stdout).expect("invalid JSON");
    let arr2 = json2.as_array().expect("expected array");

    // Should now have 2 events
    assert_eq!(arr2.len(), 2, "Expected 2 events after adding missing in");
    let in_event = arr2
        .iter()
        .find(|e| e["kind"] == "in")
        .expect("missing in event");
    let out_event = arr2
        .iter()
        .find(|e| e["kind"] == "out")
        .expect("missing out event");

    assert_eq!(in_event["time"].as_str().unwrap(), "09:00");
    assert_eq!(out_event["time"].as_str().unwrap(), "17:00");
}
