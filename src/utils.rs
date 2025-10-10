use chrono::{Datelike, NaiveDate, NaiveDateTime, ParseError, Weekday};
use std::io;
use std::path::{Path, PathBuf};

/// Convert a `NaiveDate` into an ISO 8601 string (YYYY-MM-DD)
pub fn date2iso(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Convert an ISO 8601 string (YYYY-MM-DD) into a `NaiveDate` (strict check)
pub fn iso2date(s: &str) -> Result<NaiveDate, ParseError> {
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
    // round-trip check: must exactly match the input
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

/// Returns the day of the week in various formats.
/// - `type_wd = 's'` → short, e.g. "Mo"
/// - `type_wd = 'm'` → medium, e.g. "Mon"
/// - `type_wd = 'l'` → long, e.g. "Monday"
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
        String::new() // if the date is invalid, return an empty string
    }
}
pub fn parse_work_duration_to_minutes(s: &str) -> i64 {
    // Accepts: "8h", "7h 36m", "7h36m", "  6h   15m ", "45m"
    let cleaned = s.trim().to_lowercase();
    let mut hours: i64 = 0;
    let mut minutes: i64 = 0;

    // parsing without regex: number followed by 'h' or 'm'
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
            // separator: discard orphan numbers
            if !num.is_empty() {
                num.clear();
            }
        }
    }
    hours * 60 + minutes
}

/// Convert minutes into a "HH:MM" formatted string
pub fn mins2hhmm(minutes: i32, splitted: Option<bool>) -> Result<String, (String, String)> {
    // default = false
    let splitted = splitted.unwrap_or(false);

    let hours = minutes / 60;
    let mins = minutes % 60;

    if splitted {
        // return tuple (HH, MM) as Err to distinguish
        Err((format!("{:02}", hours), format!("{:02}", mins)))
    } else {
        // return string "HH:MM"
        Ok(format!("{:02}:{:02}", hours, mins))
    }
}

/// Converts total minutes to a human-readable (HH, MM) tuple.
/// Always returns positive values (absolute duration).
pub fn mins2readable(minutes: i32) -> (String, String) {
    let abs_minutes = minutes.abs();
    match mins2hhmm(abs_minutes, Some(true)) {
        Err((hh, mm)) => (hh, mm),
        Ok(s) => {
            let (hh, mm) = s.split_once(':').unwrap_or(("00", "00"));
            (hh.to_string(), mm.to_string())
        }
    }
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
            (label.clone(), "\x1b[0m".to_string()) // fallback without color
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

#[cfg(target_os = "windows")]
pub fn compress_backup(dest: &Path) -> io::Result<PathBuf> {
    use std::fs::File;
    use zip::{CompressionMethod, ZipWriter, write::FileOptions};

    let zip_path = dest.with_extension("zip");
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut f = File::open(dest)?;
    zip.start_file(dest.file_name().unwrap().to_string_lossy(), options)?;
    std::io::copy(&mut f, &mut zip)?;
    zip.finish()?;

    println!("✅ Compressed backup: {}", zip_path.display());
    Ok(zip_path)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn compress_backup(dest: &Path) -> io::Result<PathBuf> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::fs::File;
    use tar::Builder;

    let tar_gz_path = dest.with_extension("tar.gz");
    let tar_gz = File::create(&tar_gz_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    tar.append_path_with_name(dest, dest.file_name().unwrap())?;
    tar.finish()?;

    println!("✅ Compressed backup: {}", tar_gz_path.display());
    Ok(tar_gz_path)
}
