use chrono::{Duration, NaiveTime};

/// Calculate the expected exit time.
///
/// Rules:
/// - Standard working time: 8 hours
/// - Lunch break: minimum 30 minutes (included in the calculation)
/// - If the break is longer than 30 minutes, the extra time must be recovered
///   by leaving later the same day
/// - Maximum lunch break allowed: 90 minutes
pub fn calculate_expected_exit(start: &str, lunch: i32) -> NaiveTime {
    let start_time = NaiveTime::parse_from_str(start, "%H:%M").expect("Invalid start time format");
    let work_duration = Duration::hours(8);

    // Clamp lunch to [30, 90]
    let lunch_clamped = lunch.clamp(30, 90);

    // Always count 30 min, add extra if >30
    let base_lunch = Duration::minutes(30);
    let extra = Duration::minutes((lunch_clamped - 30) as i64);

    start_time + work_duration + base_lunch + extra
}

/// Calculate surplus time (actual exit - expected exit)
pub fn calculate_surplus(start: &str, lunch: i32, end: &str) -> Duration {
    let expected = calculate_expected_exit(start, lunch);
    let actual = NaiveTime::parse_from_str(end, "%H:%M").expect("Invalid end time format");

    actual - expected
}
