#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use r_timelog::utils::{date2iso, datetime2iso, iso2date, iso2datetime};

    #[test]
    fn test_date2iso_and_iso2date() {
        let date = NaiveDate::from_ymd_opt(2025, 9, 21).unwrap();
        let iso = date2iso(&date);
        assert_eq!(iso, "2025-09-21");

        let parsed = iso2date(&iso).unwrap();
        assert_eq!(parsed, date);
    }

    #[test]
    fn test_datetime2iso_and_iso2datetime() {
        let dt = NaiveDateTime::parse_from_str("2025-09-21 14:35:50", "%Y-%m-%d %H:%M:%S").unwrap();
        let iso = datetime2iso(&dt);
        assert_eq!(iso, "2025-09-21 14:35:50");

        let parsed = iso2datetime(&iso).unwrap();
        assert_eq!(parsed, dt);
    }

    #[test]
    fn test_invalid_date_string() {
        let invalid = "2025-9-21"; // formato errato
        assert!(iso2date(invalid).is_err());
    }

    #[test]
    fn test_invalid_datetime_string() {
        let invalid = "2025-09-21 14:35"; // mancano i secondi
        assert!(iso2datetime(invalid).is_err());
    }
}
