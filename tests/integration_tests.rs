use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::env;
use std::path::PathBuf;

/// Create a unique test DB path inside the system temp dir
fn setup_test_db(name: &str) -> String {
    // Cross-platform: /tmp su Linux/macOS, %TEMP% su Windows
    let mut path: PathBuf = env::temp_dir();
    path.push(format!("{}_rtimelog.sqlite", name));

    let db_path = path.to_string_lossy().to_string();

    // Rimuove il file se esiste già (reset)
    std::fs::remove_file(&db_path).ok();

    db_path
}

#[test]
fn test_list_sessions_all() {
    let db_path = setup_test_db("all");

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-08-31",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-15",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2024-09-10",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list"])
        .assert()
        .success()
        .stdout(contains("2025-08-31"))
        .stdout(contains("2025-09-15"))
        .stdout(contains("2024-09-10"));
}

#[test]
fn test_list_sessions_filter_year() {
    let db_path = setup_test_db("year");

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-01-10",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-05-20",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2024-12-31",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list", "--period", "2025"])
        .assert()
        .success()
        .stdout(contains("2025-01-10"))
        .stdout(contains("2025-05-20"))
        .stdout(contains("📅 Saved sessions for year 2025:"))
        .stdout(
            predicates::str::is_match("2024-12-31")
                .expect("Invalid regex")
                .not(),
        );
}

#[test]
fn test_list_sessions_filter_year_month() {
    let db_path = setup_test_db("year_month");

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-01",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-15",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-10-01",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2024-09-01",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list", "--period", "2025-09"])
        .assert()
        .success()
        .stdout(contains("2025-09-01"))
        .stdout(contains("2025-09-15"))
        .stdout(contains("📅 Saved sessions for September 2025:"))
        .stdout(
            predicates::str::is_match("2025-10-01")
                .expect("Invalid regex")
                .not(),
        )
        .stdout(
            predicates::str::is_match("2024-09-01")
                .expect("Invalid regex")
                .not(),
        );
}

#[test]
fn test_list_sessions_invalid_period() {
    let db_path = setup_test_db("invalid_period");

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-01",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list", "--period", "2025-9"])
        .assert()
        .failure()
        .stderr(contains("InvalidQuery"));
}

#[test]
fn test_add_and_list_with_company_position() {
    let db_path = setup_test_db("with_company_position");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add a session in company mode (A), crossing lunch window but without specifying lunch
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-14",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    // List should show Pos A and Lunch 30 min (auto-applied)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list"])
        .assert()
        .success()
        .stdout(contains("Position O"))
        .stdout(contains("Lunch 00:30"))
        .stdout(contains("Expected"))
        .stdout(contains("Surplus"));
}

#[test]
fn test_add_and_list_with_remote_position_lunch_zero() {
    let db_path = setup_test_db("with_remote_position_lunch_zero");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add a session in remote mode (R), crossing lunch window, no lunch specified
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-15",
            "R",
            "09:00",
            "0",
            "17:00",
        ])
        .assert()
        .success();

    // List should show Pos R and Lunch 0 min (allowed)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list"])
        .assert()
        .success()
        .stdout(contains("Position R"))
        .stdout(contains("Lunch   -"));
}

#[test]
fn test_add_and_list_incomplete_session() {
    let db_path = setup_test_db("incomplete_session");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add only start time (no end)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "add", "2025-09-16", "O", "09:00"])
        .assert()
        .success();

    // List should show Pos A and Start 09:00 but End "-"
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list"])
        .assert()
        .success()
        .stdout(contains("Position O"))
        .stdout(contains("Start 09:00"))
        .stdout(contains("End   -"));
}

#[test]
fn test_add_and_list_holiday_position() {
    let db_path = setup_test_db("holiday_position");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Adding a day with Holiday position
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-09-21",
            "--pos",
            "H",
        ])
        .assert()
        .success()
        .stdout(contains("Holiday registered"));

    // List should show 'Holiday' as position and no more data's
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list"])
        .assert()
        .success()
        .stdout(contains("Holiday"));
}
