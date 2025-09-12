use chrono::{Duration, NaiveTime};
use r_timelog::logic;
use r_timelog::logic::month_name;

#[test]
fn test_expected_exit_with_min_lunch() {
    let start = "09:00";
    let lunch = 30;
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:30", "%H:%M").unwrap()
    );
}

#[test]
fn test_expected_exit_with_short_lunch_treated_as_min() {
    let start = "09:00";
    let lunch = 15; // less than 30, treated as 30
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:30", "%H:%M").unwrap()
    );
}

#[test]
fn test_expected_exit_with_extra_lunch() {
    let start = "09:00";
    let lunch = 45; // 30 + 15 extra → recover in exit
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:45", "%H:%M").unwrap()
    );
}

#[test]
fn test_expected_exit_with_max_lunch_capped() {
    let start = "09:00";
    let lunch = 120; // capped to 90
    let expected = logic::calculate_expected_exit(start, lunch);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("18:30", "%H:%M").unwrap()
    );
}

#[test]
fn test_surplus_exact_time() {
    let surplus = logic::calculate_surplus("09:00", 30, "17:30");
    assert_eq!(surplus, Duration::zero());
}

#[test]
fn test_surplus_overtime() {
    let surplus = logic::calculate_surplus("09:00", 60, "18:30");
    assert_eq!(surplus, Duration::minutes(30));
}

#[test]
fn test_surplus_leave_early() {
    let surplus = logic::calculate_surplus("09:00", 90, "18:00");
    assert_eq!(surplus, Duration::minutes(-30));
}

#[test]
fn test_month_name_valid() {
    assert_eq!(month_name("01"), "January");
    assert_eq!(month_name("02"), "February");
    assert_eq!(month_name("03"), "March");
    assert_eq!(month_name("04"), "April");
    assert_eq!(month_name("05"), "May");
    assert_eq!(month_name("06"), "June");
    assert_eq!(month_name("07"), "July");
    assert_eq!(month_name("08"), "August");
    assert_eq!(month_name("09"), "September");
    assert_eq!(month_name("10"), "October");
    assert_eq!(month_name("11"), "November");
    assert_eq!(month_name("12"), "December");
}

#[test]
fn test_month_name_invalid() {
    assert_eq!(month_name("00"), "Unknown");
    assert_eq!(month_name("13"), "Unknown");
    assert_eq!(month_name("xx"), "Unknown");
}
