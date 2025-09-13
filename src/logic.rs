use chrono::{Duration, NaiveTime};

pub fn month_name(month: &str) -> &'static str {
    match month {
        "01" => "January",
        "02" => "February",
        "03" => "March",
        "04" => "April",
        "05" => "May",
        "06" => "June",
        "07" => "July",
        "08" => "August",
        "09" => "September",
        "10" => "October",
        "11" => "November",
        "12" => "December",
        _ => "Unknown",
    }
}

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

/// Check if the work interval crosses the lunch window (12:30–14:30).
pub fn crosses_lunch_window(start: &str, end: &str) -> bool {
    let start_time = NaiveTime::parse_from_str(start, "%H:%M");
    let end_time = NaiveTime::parse_from_str(end, "%H:%M");

    if start_time.is_err() || end_time.is_err() {
        return false;
    }

    let start_time = start_time.unwrap();
    let end_time = end_time.unwrap();

    let lunch_window_start = NaiveTime::parse_from_str("12:30", "%H:%M").unwrap();
    let lunch_window_end = NaiveTime::parse_from_str("14:30", "%H:%M").unwrap();

    start_time < lunch_window_end && end_time > lunch_window_start
}
