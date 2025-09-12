use r_timelog::db;
use rusqlite::Connection;

#[test]
fn test_list_sessions_all() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-08-31", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-09-15", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2024-09-10", "09:00", 30, "17:00").unwrap();

    let sessions = db::list_sessions(&conn, None).unwrap();
    assert_eq!(sessions.len(), 3); // tutti
}

#[test]
fn test_list_sessions_filter_year() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    db::add_session(&conn, "2025-01-10", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-05-20", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2024-12-31", "09:00", 30, "17:00").unwrap();

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

    db::add_session(&conn, "2025-09-01", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-09-15", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2025-10-01", "09:00", 30, "17:00").unwrap();
    db::add_session(&conn, "2024-09-01", "09:00", 30, "17:00").unwrap();

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

    db::add_session(&conn, "2025-09-01", "09:00", 30, "17:00").unwrap();

    let result = db::list_sessions(&conn, Some("2025-9")); // formato errato
    assert!(result.is_err());
}
