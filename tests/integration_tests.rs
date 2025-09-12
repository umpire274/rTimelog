use chrono::{Duration, NaiveTime};
use rusqlite::Connection;

// Import your modules
use r_timelog::db;
use r_timelog::logic;

#[test]
fn test_db_and_logic_integration_min_lunch() {
    // In-memory database
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    // Insert: start 09:00, lunch 30, end 17:30
    db::add_session(&conn, "2025-09-12", "09:00", 30, "17:30").unwrap();

    let sessions = db::list_sessions(&conn).unwrap();
    let s = &sessions[0];

    // Expected exit: 17:30 (09:00 + 8h + 30m)
    let expected = logic::calculate_expected_exit(&s.start, s.lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:30", "%H:%M").unwrap()
    );

    // Surplus: 17:30 - 17:30 = 0
    let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
    assert_eq!(surplus, Duration::zero());
}

#[test]
fn test_db_and_logic_integration_extra_lunch() {
    // In-memory database
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    // Insert: start 09:00, lunch 45, end 18:00
    db::add_session(&conn, "2025-09-13", "09:00", 45, "18:00").unwrap();

    let sessions = db::list_sessions(&conn).unwrap();
    let s = &sessions[0];

    // Expected exit: 17:45 (09:00 + 8h + 30m + 15m extra)
    let expected = logic::calculate_expected_exit(&s.start, s.lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:45", "%H:%M").unwrap()
    );

    // Surplus: 18:00 - 17:45 = +15 min
    let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
    assert_eq!(surplus, Duration::minutes(15));
}

#[test]
fn test_db_and_logic_integration_long_lunch_capped() {
    // In-memory database
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    // Insert: start 09:00, lunch 120 (too long, capped at 90), end 18:30
    db::add_session(&conn, "2025-09-14", "09:00", 120, "18:30").unwrap();

    let sessions = db::list_sessions(&conn).unwrap();
    let s = &sessions[0];

    // Expected exit: 18:30 (09:00 + 8h + 30m + 60m extra capped)
    let expected = logic::calculate_expected_exit(&s.start, s.lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("18:30", "%H:%M").unwrap()
    );

    // Surplus: 18:30 - 18:30 = 0
    let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
    assert_eq!(surplus, Duration::zero());
}
