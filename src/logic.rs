use chrono::{Duration, NaiveTime};

/// Calculate the expected exit time.
///
/// Rules:
/// - Standard working time: 8 hours
/// - Maximum lunch break: 1 hour
/// - If the lunch break is shorter than 60 minutes, still add 1 hour to the total,
///   subtracting the difference (meaning the employee can leave earlier).
pub fn calculate_expected_exit(start: &str, lunch: i32) -> NaiveTime {
    let start_time = NaiveTime::parse_from_str(start, "%H:%M").expect("Invalid start time format");

    // 8 hours of work
    let work_duration = Duration::hours(8);

    // lunch break: max 60 minutes
    let pause = if lunch > 60 { 60 } else { lunch };

    start_time + work_duration + Duration::minutes(pause as i64)
}

/// Calculate surplus time (actual exit - expected exit)
pub fn calculate_surplus(start: &str, lunch: i32, end: &str) -> Duration {
    let expected = calculate_expected_exit(start, lunch);
    let actual = NaiveTime::parse_from_str(end, "%H:%M").expect("Invalid end time format");

    actual - expected
}
