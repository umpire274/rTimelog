use r_timelog::db;
use r_timelog::logic;
use rusqlite::Connection;

#[test]
fn test_surplus_zero_output() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-09-12", "09:00", 30, "17:30").unwrap();
    let sessions = db::list_sessions(&conn).unwrap();
    let s = &sessions[0];

    let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
    let surplus_minutes = surplus.num_minutes();

    // Zero should display as "0"
    let formatted = if surplus_minutes == 0 {
        "0".to_string()
    } else {
        format!("{:+}", surplus_minutes)
    };
    assert_eq!(formatted, "0");
}

#[test]
fn test_surplus_positive_output() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-09-13", "09:00", 60, "18:30").unwrap();
    let sessions = db::list_sessions(&conn).unwrap();
    let s = &sessions[0];

    let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
    let surplus_minutes = surplus.num_minutes();

    // Positive should display with a "+"
    let formatted = format!("{:+}", surplus_minutes);
    assert_eq!(formatted, "+30");
}

#[test]
fn test_surplus_negative_output() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-09-14", "09:00", 60, "17:30").unwrap();
    let sessions = db::list_sessions(&conn).unwrap();
    let s = &sessions[0];

    let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
    let surplus_minutes = surplus.num_minutes();

    // Negative should display with "-"
    let formatted = format!("{:+}", surplus_minutes);
    assert_eq!(formatted, "-30");
}
