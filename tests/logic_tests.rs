use chrono::{NaiveTime, Duration};
use r_timelog::logic; // Assumes you declare `pub mod logic;` in lib.rs or main.rs

#[test]
fn test_expected_exit_with_full_lunch() {
    let start = "09:00";
    let lunch = 60;
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(expected, NaiveTime::parse_from_str("18:00", "%H:%M").unwrap());
}

#[test]
fn test_expected_exit_with_short_lunch() {
    let start = "09:00";
    let lunch = 30; // 30 minutes, but rule forces 1 hour
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(expected, NaiveTime::parse_from_str("17:30", "%H:%M").unwrap());
}

#[test]
fn test_expected_exit_with_long_lunch() {
    let start = "09:00";
    let lunch = 120; // capped at 60
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(expected, NaiveTime::parse_from_str("18:00", "%H:%M").unwrap());
}

#[test]
fn test_surplus_positive() {
    let surplus = logic::calculate_surplus("09:00", 60, "18:30");
    assert_eq!(surplus, Duration::minutes(30)); // 30 minutes overtime
}

#[test]
fn test_surplus_negative() {
    let surplus = logic::calculate_surplus("09:00", 60, "17:30");
    assert_eq!(surplus, Duration::minutes(-30)); // left 30 minutes earlier
}

#[test]
fn test_surplus_zero() {
    let surplus = logic::calculate_surplus("09:00", 60, "18:00");
    assert_eq!(surplus, Duration::zero()); // left exactly on time
}
