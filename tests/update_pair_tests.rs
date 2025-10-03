use assert_cmd::Command;
use serde_json::Value;
use std::env;
use std::path::PathBuf;

/// Create a unique test DB path inside the system temp dir
fn setup_test_db(name: &str) -> String {
    let mut path: PathBuf = env::temp_dir();
    path.push(format!("{}_rtimelog.sqlite", name));
    let db_path = path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&db_path);
    db_path
}

#[test]
fn test_update_does_not_create_new_pair() {
    let db_path = setup_test_db("update_pair");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add initial full session -> produces exactly 2 events (in/out) with pair=1
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-20",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    // Capture events JSON after initial insert
    let initial_output = Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events", "--json"])
        .output()
        .expect("failed to list events (initial)");
    assert!(initial_output.status.success());
    let initial_json: Value =
        serde_json::from_slice(&initial_output.stdout).expect("invalid JSON initial events");
    let initial_events = initial_json.as_array().expect("expected array of events");
    assert_eq!(
        initial_events.len(),
        2,
        "Expected exactly 2 events after first add"
    );

    let init_in = initial_events
        .iter()
        .find(|e| e["kind"] == "in")
        .expect("missing in event");
    let init_out = initial_events
        .iter()
        .find(|e| e["kind"] == "out")
        .expect("missing out event");

    let in_id = init_in["id"].as_i64().unwrap();
    let out_id = init_out["id"].as_i64().unwrap();

    // Update ONLY start time via explicit edit (pair 1)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-20",
            "--edit",
            "--pair",
            "1",
            "--in",
            "09:15",
        ])
        .assert()
        .success();

    // Update ONLY end time via explicit edit (pair 1)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-20",
            "--edit",
            "--pair",
            "1",
            "--out",
            "17:05",
        ])
        .assert()
        .success();

    // Update ONLY lunch via explicit edit (pair 1)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-20",
            "--edit",
            "--pair",
            "1",
            "--lunch",
            "45",
        ])
        .assert()
        .success();

    // Re-capture events JSON after updates
    let final_output = Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events", "--json"])
        .output()
        .expect("failed to list events (final)");
    assert!(final_output.status.success());
    let final_json: Value =
        serde_json::from_slice(&final_output.stdout).expect("invalid JSON final events");
    let final_events = final_json.as_array().expect("expected array of events");

    // Still exactly 2 events
    assert_eq!(
        final_events.len(),
        2,
        "Editing fields must NOT create extra events/pairs"
    );

    // Fetch updated events
    let final_in = final_events
        .iter()
        .find(|e| e["kind"] == "in")
        .expect("missing final in event");
    let final_out = final_events
        .iter()
        .find(|e| e["kind"] == "out")
        .expect("missing final out event");

    // IDs must be unchanged (no insertion of new rows)
    assert_eq!(
        final_in["id"].as_i64().unwrap(),
        in_id,
        "In event id changed unexpectedly (new event created?)"
    );
    assert_eq!(
        final_out["id"].as_i64().unwrap(),
        out_id,
        "Out event id changed unexpectedly (new event created?)"
    );

    // Times updated
    assert_eq!(
        final_in["time"].as_str().unwrap(),
        "09:15",
        "Start time not updated in place"
    );
    assert_eq!(
        final_out["time"].as_str().unwrap(),
        "17:05",
        "End time not updated in place"
    );

    // Lunch updated only on out event
    assert_eq!(
        final_in["lunch_break"].as_i64().unwrap(),
        0,
        "In event lunch should remain 0"
    );
    assert_eq!(
        final_out["lunch_break"].as_i64().unwrap(),
        45,
        "Out event lunch not updated"
    );

    // Pair logic unchanged: both pair=1, unmatched=false
    assert_eq!(
        final_in["pair"].as_u64().unwrap(),
        1,
        "In event pair id changed"
    );
    assert_eq!(
        final_out["pair"].as_u64().unwrap(),
        1,
        "Out event pair id changed"
    );
    assert!(
        !final_in["unmatched"].as_bool().unwrap(),
        "In event became unmatched unexpectedly"
    );
    assert!(
        !final_out["unmatched"].as_bool().unwrap(),
        "Out event became unmatched unexpectedly"
    );
}
