use chrono::{Datelike, NaiveDate, NaiveDateTime, ParseError, Weekday};

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

/// Returns the day of the week in various formats...
/// - `type_wd = 's'` → short, es. "Mo"
/// - `type_wd = 'm'` → medium, es. "Mon"
/// - `type_wd = 'l'` → long, es. "Monday"
pub fn weekday_str(date_str: &str, type_wd: char) -> String {
    if let Ok(ndate) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let wd = ndate.weekday();
        match type_wd {
            's' => match wd {
                Weekday::Mon => "Mo",
                Weekday::Tue => "Tu",
                Weekday::Wed => "We",
                Weekday::Thu => "Th",
                Weekday::Fri => "Fr",
                Weekday::Sat => "Sa",
                Weekday::Sun => "Su",
            }
            .to_string(),
            'l' => match wd {
                Weekday::Mon => "Monday",
                Weekday::Tue => "Tuesday",
                Weekday::Wed => "Wednesday",
                Weekday::Thu => "Thursday",
                Weekday::Fri => "Friday",
                Weekday::Sat => "Saturday",
                Weekday::Sun => "Sunday",
            }
            .to_string(),
            // default → medium
            _ => match wd {
                Weekday::Mon => "Mon",
                Weekday::Tue => "Tue",
                Weekday::Wed => "Wed",
                Weekday::Thu => "Thu",
                Weekday::Fri => "Fri",
                Weekday::Sat => "Sat",
                Weekday::Sun => "Sun",
            }
            .to_string(),
        }
    } else {
        String::new() // se la data non è valida, restituisce stringa vuota
    }
}
pub fn parse_work_duration_to_minutes(s: &str) -> i64 {
    // Accetta: "8h", "7h 36m", "7h36m", "  6h   15m ", "45m"
    let cleaned = s.trim().to_lowercase();
    let mut hours: i64 = 0;
    let mut minutes: i64 = 0;

    // parsing senza regex: numero seguito da 'h' o 'm'
    let mut num = String::new();
    for ch in cleaned.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else if ch == 'h' {
            if let Ok(h) = num.parse::<i64>() {
                hours = h;
            }
            num.clear();
        } else if ch == 'm' {
            if let Ok(m) = num.parse::<i64>() {
                minutes = m;
            }
            num.clear();
        } else {
            // separatore: scarta numeri orfani
            if !num.is_empty() {
                num.clear();
            }
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

/// Return a tuple (label, colorized_label) for a given working position.
/// O = Office (blue), R = Remote (cyan), F = On-site (yellow), H = Holiday (purple background).
pub fn describe_position(pos: &str) -> (String, String) {
    match pos {
        "O" => {
            let label = "Office".to_string();
            let colored = "\x1b[34m".to_string();
            (label, colored)
        }
        "R" => {
            let label = "Remote".to_string();
            let colored = "\x1b[36m".to_string();
            (label, colored)
        }
        "C" => {
            let label = "On-site (Client)".to_string();
            let colored = "\x1b[33m".to_string();
            (label, colored)
        }
        "H" => {
            let label = "Holiday".to_string();
            let colored = "\x1b[45;97;1m".to_string();
            (label, colored)
        }
        "M" => {
            let label = "Mixed".to_string();
            let colored = "\x1b[35m".to_string(); // magenta
            (label, colored)
        }
        _ => {
            let label = pos.to_string();
            (label.clone(), "\x1b[0m".to_string()) // fallback senza colore
        }
    }
}

/// Return true if the given date (YYYY-MM-DD) is the last day of its month.
/// Returns false if the date cannot be parsed.
pub fn is_last_day_of_month(date_str: &str) -> bool {
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => {
            // compute first day of next month
            let (y, m) = (d.year(), d.month());
            let next_month_first = if m == 12 {
                NaiveDate::from_ymd_opt(y + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(y, m + 1, 1)
            };
            if let Some(next_first) = next_month_first {
                let last_day = next_first - chrono::Duration::days(1);
                d == last_day
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
