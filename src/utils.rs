use chrono::{NaiveDate, NaiveDateTime, ParseError};

/// Convert a `NaiveDate` into an ISO 8601 string (YYYY-MM-DD)
pub fn date2iso(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Convert an ISO 8601 string (YYYY-MM-DD) into a `NaiveDate` (strict check)
pub fn iso2date(s: &str) -> Result<NaiveDate, ParseError> {
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
    // round-trip check: deve coincidere esattamente con l’input
    if date2iso(&date) == s {
        Ok(date)
    } else {
        NaiveDate::parse_from_str("xxxx-xx-xx", "%Y-%m-%d")
    }
}

/// Convert a `NaiveDateTime` into an ISO 8601 string (YYYY-MM-DD HH:MM:SS)
pub fn datetime2iso(dt: &NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Convert an ISO 8601 string (YYYY-MM-DD HH:MM:SS) into a `NaiveDateTime` (strict check)
pub fn iso2datetime(s: &str) -> Result<NaiveDateTime, ParseError> {
    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    // round-trip check
    if datetime2iso(&dt) == s {
        Ok(dt)
    } else {
        NaiveDateTime::parse_from_str("xxxx-xx-xx xx:xx:xx", "%Y-%m-%d %H:%M:%S")
    }
}

pub fn parse_work_duration_to_minutes(s: &str) -> i64 {
    let mut hours = 0;
    let mut minutes = 0;

    let parts: Vec<&str> = s.split_whitespace().collect();
    for part in parts {
        if part.ends_with('h') {
            if let Ok(h) = part.trim_end_matches('h').parse::<i64>() {
                hours = h;
            }
        } else if part.ends_with('m')
            && let Ok(m) = part.trim_end_matches('m').parse::<i64>()
        {
            minutes = m;
        }
    }

    hours * 60 + minutes
}

/// Convert minutes into a "HH:MM" formatted string
pub fn mins2hhmm(minutes: i32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    format!("{:02}:{:02}", hours, mins)
}

/// Generate a separator string with `width` repetitions of the given `ch`,
/// aligned to the given column (`align`).
pub fn make_separator(ch: char, width: usize, align: usize) -> String {
    let line = ch.to_string().repeat(width);
    format!("{:>align$}", line, align = align)
}

/// Print a separator line with `width` repetitions of the given `ch`,
/// aligned to the given column (`align`).
pub fn print_separator(ch: char, width: usize, align: usize) {
    println!("{}", make_separator(ch, width, align));
}
