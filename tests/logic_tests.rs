use chrono::{Duration, NaiveTime};
use rtimelogger::config::Config;
use rtimelogger::logic::{calculate_expected_exit, calculate_surplus, month_name};
use rtimelogger::utils;
use rtimelogger::utils::mins2hhmm;

#[test]
fn test_expected_exit_basic() {
    let config = Config::default();

    let start = "08:15";
    let work_minutes = 456; // 7h36m
    let lunch = 30; // minimo

    let expected = calculate_expected_exit(start, work_minutes, lunch, &config);
    assert_eq!(expected.format("%H:%M").to_string(), "16:21"); // esempio
}

#[test]
fn test_expected_exit_with_min_lunch() {
    let config = Config::default();

    let start = "09:00";
    let lunch = 30;
    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let expected = calculate_expected_exit(start, work_minutes, lunch, &config);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:30", "%H:%M").unwrap()
    );
}

#[test]
fn test_expected_exit_with_short_lunch_treated_as_min() {
    let config = Config::default();

    let start = "09:00";
    let lunch = 15; // less than 30, treated as 30
    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let expected = calculate_expected_exit(start, work_minutes, lunch, &config);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:30", "%H:%M").unwrap()
    );
}

#[test]
fn test_expected_exit_with_extra_lunch() {
    let config = Config::default();

    let start = "09:00";
    let lunch = 45; // 30 + 15 extra → recover in exit
    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let expected = calculate_expected_exit(start, work_minutes, lunch, &config);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("17:45", "%H:%M").unwrap()
    );
}

#[test]
fn test_expected_exit_with_max_lunch_capped() {
    let config = Config::default();

    let start = "09:00";
    let lunch = 120; // capped to 90
    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let expected = calculate_expected_exit(start, work_minutes, lunch, &config);
    assert_eq!(
        expected,
        NaiveTime::parse_from_str("18:30", "%H:%M").unwrap()
    );
}

#[test]
fn test_surplus_exact_time() {
    let config = Config::default();

    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let surplus = calculate_surplus("09:00", 30, "17:30", work_minutes, &config);
    assert_eq!(surplus, Duration::zero());
}

#[test]
fn test_surplus_overtime() {
    let config = Config::default();

    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let surplus = calculate_surplus("09:00", 30, "18:00", work_minutes, &config);
    assert_eq!(surplus, Duration::minutes(30));
}

#[test]
fn test_surplus_leave_early() {
    let config = Config::default();

    let work_minutes = utils::parse_work_duration_to_minutes("8h");
    let surplus = calculate_surplus("09:00", 30, "17:00", work_minutes, &config);
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

#[test]
fn test_exact_hours() {
    assert_eq!(mins2hhmm(0), "00:00");
    assert_eq!(mins2hhmm(60), "01:00");
    assert_eq!(mins2hhmm(120), "02:00");
}

#[test]
fn test_only_minutes() {
    assert_eq!(mins2hhmm(5), "00:05");
    assert_eq!(mins2hhmm(30), "00:30");
    assert_eq!(mins2hhmm(59), "00:59");
}

#[test]
fn test_hours_and_minutes() {
    assert_eq!(mins2hhmm(75), "01:15");
    assert_eq!(mins2hhmm(135), "02:15");
    assert_eq!(mins2hhmm(1439), "23:59"); // limite di una giornata
}
