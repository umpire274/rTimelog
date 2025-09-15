use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::path::PathBuf;

/// Create a unique test DB path inside the system temp dir
fn setup_test_db(name: &str) -> String {
    let mut path = PathBuf::new();
    if cfg!(target_os = "windows") {
        path.push("C:\\Windows\\Temp\\");
    } else {
        path.push("/tmp/");
    }
    path.push(format!("{}_rtimelog.sqlite", name));
    let db_path = path.to_string_lossy().to_string();
    std::fs::remove_file(&db_path).ok(); // reset if exists
    db_path
}

#[test]
fn test_list_sessions_all() {
    let db_path = setup_test_db("all");
    println!("test_list_sessions_all() Using test DB: {}", db_path);

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
    println!(
        "test_list_sessions_filter_year() Using test DB: {}",
        db_path
    );

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
    println!(
        "test_list_sessions_filter_year_month() Using test DB: {}",
        db_path
    );

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
    println!(
        "test_list_sessions_invalid_period() Using test DB: {}",
        db_path
    );

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
    println!(
        "test_add_and_list_with_company_position() Using test DB: {}",
        db_path
    );

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
        .stdout(contains("Lunch 30 min"))
        .stdout(contains("Expected"))
        .stdout(contains("Surplus"));
}

#[test]
fn test_add_and_list_with_remote_position_lunch_zero() {
    let db_path = setup_test_db("with_remote_position_lunch_zero");
    println!(
        "test_add_and_list_with_remote_position_lunch_zero() Using test DB: {}",
        db_path
    );

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
    println!(
        "test_add_and_list_incomplete_session() Using test DB: {}",
        db_path
    );

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "init"])
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
