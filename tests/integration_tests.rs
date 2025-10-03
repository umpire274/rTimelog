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
fn test_list_sessions_filter_position() {
    let db_path = setup_test_db("filter_position");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add Office (O)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-09-10",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    // Add Remote (R)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-09-11",
            "R",
            "09:15",
            "0",
            "17:15",
        ])
        .assert()
        .success();

    // Add Holiday (H)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "add", "2025-09-12", "H"])
        .assert()
        .success();

    // Filter O
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--pos", "O"])
        .assert()
        .success()
        .stdout(contains("2025-09-10"))
        .stdout(contains("Office"))
        .stdout(contains("2025-09-11").not())
        .stdout(contains("2025-09-12").not());

    // Filter R
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--pos", "R"])
        .assert()
        .success()
        .stdout(contains("2025-09-11"))
        .stdout(contains("Remote"))
        .stdout(contains("2025-09-10").not())
        .stdout(contains("2025-09-12").not());

    // Filter H
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--pos", "H"])
        .assert()
        .success()
        .stdout(contains("2025-09-12"))
        .stdout(contains("Holiday"))
        .stdout(contains("2025-09-10").not())
        .stdout(contains("2025-09-11").not());
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
        .stdout(contains("Office"))
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
        .stdout(contains("Remote"))
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
        .stdout(contains("Office"))
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
        .stdout(contains("Position Holiday"));

    // List should show 'Holiday' as position and no more data's
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list"])
        .assert()
        .success()
        .stdout(contains("Holiday"));
}

#[test]
fn test_list_sessions_positions_with_colors() {
    // (Position, Label atteso, Codice ANSI atteso)
    let cases = vec![
        ("O", "Office", "\x1b[34m"),           // Office → blu
        ("R", "Remote", "\x1b[36m"),           // Remote → ciano
        ("C", "On-site (Client)", "\x1b[33m"), // Client → giallo
        ("H", "Holiday", "\x1b[45;97;1m"),     // Holiday → viola bg + bold
    ];

    for (pos, label, color) in cases {
        let db_path = setup_test_db(&format!("pos_{}", pos));

        // Init DB
        Command::cargo_bin("rtimelog")
            .unwrap()
            .args(["--db", &db_path, "--test", "init"])
            .assert()
            .success();

        // Add session (Holiday non ha start/end, le altre sì)
        let mut args = vec!["--db", &db_path, "--test", "add", "2025-09-15", pos];
        if pos != "H" {
            args.extend(&["09:00", "30", "17:00"]);
        }

        Command::cargo_bin("rtimelog")
            .unwrap()
            .args(&args)
            .assert()
            .success();

        // List filtrato per posizione → deve contenere label e colore
        Command::cargo_bin("rtimelog")
            .unwrap()
            .args(["--db", &db_path, "--test", "list", "--pos", pos])
            .assert()
            .success()
            .stdout(contains(label))
            .stdout(contains(color));
    }
}

#[test]
fn test_add_and_delete_session() {
    let db_path = setup_test_db("delete_session");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add a session
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-09-20",
            "O",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    // Verify session is listed
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list"])
        .assert()
        .success()
        .stdout(contains("2025-09-20"));

    // Delete by date (new behavior) -- answer 'y' to confirmation prompt
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "del", "2025-09-20"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(contains("Deleted").or(contains("deleted")));

    // Verify session no longer appears in list
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list"])
        .assert()
        .success()
        .stdout(contains("2025-09-20").not());
}

#[test]
fn test_delete_nonexistent_session() {
    let db_path = setup_test_db("delete_nonexistent");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Try to delete a date that does not exist: confirm with 'y' and expect 0 rows deleted
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "del", "2099-01-01"])
        .write_stdin("y\n")
        .assert()
        .success() // the command should not error
        .stdout(
            contains("Deleted 0 event(s) and 0 work_session(s)").or(contains("Deleted 0 event(s)")),
        );
}

#[test]
fn test_separator_after_month_end() {
    let db_path = setup_test_db("separator_month_end");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add last day of September and first day of October
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "add",
            "2025-09-30",
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

    // List and assert separator (25 '-' characters) is present after the 2025-09-30 line
    let sep25 = "-".repeat(25);

    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "list"])
        .assert()
        .success()
        .stdout(contains("2025-09-30"))
        .stdout(contains(sep25));
}

#[test]
fn test_list_events_filter_position_case_insensitive() {
    let db_path = setup_test_db("events_pos_case");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add Remote (R) session which creates two events (in/out)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-09-21",
            "R",
            "09:00",
            "30",
            "17:00",
        ])
        .assert()
        .success();

    // Add Office (O) session
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-09-22",
            "O",
            "09:10",
            "30",
            "17:10",
        ])
        .assert()
        .success();

    // List events filtering with lowercase 'r' to verify case-insensitive normalization
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events", "--pos", "r"])
        .assert()
        .success()
        .stdout(contains("2025-09-21")) // remote date present
        .stdout(contains("2025-09-22").not()); // office date absent
}

#[test]
fn test_events_pair_column_and_grouping() {
    let db_path = setup_test_db("events_pair_col");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Prima sessione (in/out)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-10-02",
            "R",
            "09:00",
            "30",
            "12:00",
        ])
        .assert()
        .success();

    // Seconda sessione (in/out)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-10-02",
            "R",
            "13:00",
            "0",
            "17:00",
        ])
        .assert()
        .success();

    // Lista eventi e verifica intestazione Pair e presenza dei pair id 1 e 2
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events", "--pos", "R"])
        .assert()
        .success()
        .stdout(contains("Pair"))
        .stdout(contains("  1"))
        .stdout(contains("  2"));
}

#[test]
fn test_delete_existing_pair() {
    let db_path = setup_test_db("delete_existing_pair");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Pair 1 (09:00-12:00)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-10-02",
            "R",
            "09:00",
            "30",
            "12:00",
        ])
        .assert()
        .success();

    // Pair 2 (13:00-17:00)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-10-02",
            "R",
            "13:00",
            "0",
            "17:00",
        ])
        .assert()
        .success();

    // Delete pair 1 (confirm 'y')
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "del",
            "--pair",
            "1",
            "2025-10-02",
        ])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(contains("Deleted").or(contains("deleted")));

    // List events and ensure pair1 times are gone, pair2 remains
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "list", "--events"])
        .assert()
        .success()
        .stdout(contains("13:00"))
        .stdout(contains("17:00"))
        .stdout(contains("09:00").not())
        .stdout(contains("12:00").not());
}

#[test]
fn test_delete_nonexistent_pair() {
    let db_path = setup_test_db("delete_nonexistent_pair");

    // Init DB
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args(["--db", &db_path, "--test", "init"])
        .assert()
        .success();

    // Add a single pair (so pair id 1 exists)
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "add",
            "2025-10-03",
            "O",
            "09:00",
            "30",
            "12:00",
        ])
        .assert()
        .success();

    // Try to delete a non-existent pair 5 on that date
    Command::cargo_bin("rtimelog")
        .unwrap()
        .args([
            "--db",
            &db_path,
            "--test",
            "del",
            "--pair",
            "5",
            "2025-10-03",
        ])
        .assert()
        .success()
        .stdout(contains("Pair 5 not found for date 2025-10-03"));
}
