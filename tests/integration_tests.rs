use assert_cmd::Command;
use predicates::str::contains;
use r_timelog::db;
use rusqlite::Connection;

#[test]
fn test_list_sessions_all() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-08-31", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-09-15", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2024-09-10", "A", "09:00", 30, "17:00").unwrap();

    let sessions = db::list_sessions(&conn, None).unwrap();
    assert_eq!(sessions.len(), 3); // tutti
}

#[test]
fn test_list_sessions_filter_year() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-01-10", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-05-20", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2024-12-31", "A", "09:00", 30, "17:00").unwrap();

    let sessions_2025 = db::list_sessions(&conn, Some("2025")).unwrap();
    assert_eq!(sessions_2025.len(), 2);
    for s in sessions_2025 {
        assert!(s.date.starts_with("2025"));
    }
}

#[test]
fn test_list_sessions_filter_year_month() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-09-01", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-09-15", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-10-01", "A", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2024-09-01", "A", "09:00", 30, "17:00").unwrap();

    let sessions_sep_2025 = db::list_sessions(&conn, Some("2025-09")).unwrap();
    assert_eq!(sessions_sep_2025.len(), 2);

    for s in sessions_sep_2025 {
        assert!(s.date.starts_with("2025-09"));
    }
}

#[test]
fn test_list_sessions_invalid_period() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-09-01", "A", "09:00", 30, "17:00").unwrap();

    let result = db::list_sessions(&conn, Some("2025-9")); // formato errato
    assert!(result.is_err());
}

#[test]
fn test_add_and_list_with_company_position() {
    // Clean DB
    std::fs::remove_file("worktime.sqlite").ok();

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    // Add a session in company mode (A), crossing lunch window but without specifying lunch
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["add", "2025-09-14", "A", "09:00", "0", "17:00"])
        .assert()
        .success();

    // List should show Pos A and Lunch 30 min (auto-applied)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("Position A"))
        .stdout(contains("Lunch 30 min"))
        .stdout(contains("Expected"))
        .stdout(contains("Surplus"));
}

#[test]
fn test_add_and_list_with_remote_position_lunch_zero() {
    // Clean DB
    std::fs::remove_file("worktime.sqlite").ok();

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    // Add a session in remote mode (R), crossing lunch window, no lunch specified
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["add", "2025-09-15", "R", "09:00", "0", "17:00"])
        .assert()
        .success();

    // List should show Pos R and Lunch 0 min (allowed)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("Position R"))
        .stdout(contains("Lunch   -"))
        .stdout(contains("Expected"))
        .stdout(contains("Surplus"));
}

#[test]
fn test_add_and_list_incomplete_session() {
    // Clean DB
    std::fs::remove_file("worktime.sqlite").ok();

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    // Add only start time (no end)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["add", "2025-09-16", "A", "09:00"])
        .assert()
        .success();

    // List should show Pos A and Start 09:00 but End "-"
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("Position A"))
        .stdout(contains("Start 09:00"))
        .stdout(contains("End -"));
}
