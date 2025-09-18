use crate::config::Config;
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
pub fn calculate_expected_exit(start: &str, work_minutes: i64, lunch: i32) -> NaiveTime {
    let start_time = NaiveTime::parse_from_str(start, "%H:%M").expect("Invalid start time format");
    let work_duration = Duration::minutes(work_minutes);

    // Clamp lunch to [30, 90]
    let lunch_clamped = lunch.clamp(30, 90);

    // Always count 30 min, add extra if >30
    let base_lunch = Duration::minutes(30);
    let extra = Duration::minutes((lunch_clamped - 30) as i64);

    start_time + work_duration + base_lunch + extra
}

/// Calculate surplus time (actual exit - expected exit)
pub fn calculate_surplus(start: &str, lunch: i32, end: &str, work_minutes: i64) -> Duration {
    let expected = calculate_expected_exit(start, work_minutes, lunch);
    let actual = NaiveTime::parse_from_str(end, "%H:%M").expect("Invalid end time format");

    actual - expected
}

/// Return true if the interval [start, end] overlaps the lunch window 12:30–14:30.
pub fn crosses_lunch_window(start: &str, end: &str) -> bool {
    let start_time = match NaiveTime::parse_from_str(start, "%H:%M") {
        Ok(t) => t,
        Err(_) => return false,
    };
    let end_time = match NaiveTime::parse_from_str(end, "%H:%M") {
        Ok(t) => t,
        Err(_) => return false,
    };

    let lunch_start = NaiveTime::parse_from_str("12:30", "%H:%M").unwrap();
    let lunch_end = NaiveTime::parse_from_str("14:30", "%H:%M").unwrap();

    start_time < lunch_end && end_time > lunch_start
}

/// Compute the effective lunch minutes based on position and work interval.
///
/// Rules:
/// - `A` (office): if the interval overlaps 12:30–14:30, lunch is mandatory [30..90].
///   If missing (0) it defaults to 30. Outside the window, lunch = 0.
/// - `R` (remote): lunch is optional, 0..90 accepted, even if overlapping the window.
pub fn effective_lunch_minutes(
    lunch: i32,
    start: &str,
    end: &str,
    position: char,
    config: &Config,
) -> i32 {
    let crosses = crosses_lunch_window(start, end);
    match position {
        'O' => {
            if crosses {
                let l = lunch.clamp(
                    config.min_duration_lunch_break,
                    config.max_duration_lunch_break,
                );
                if l < 30 { 30 } else { l }
            } else {
                0
            }
        }
        'H' => 0,
        _ => lunch.clamp(0, 90),
    }
}
