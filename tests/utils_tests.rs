#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use r_timelog::utils::{
        date2iso, datetime2iso, describe_position, iso2date, iso2datetime, make_separator,
    };

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

    #[test]
    fn test_make_separator_equal_signs() {
        let sep = make_separator('=', 5, 10);
        assert_eq!(sep.len(), 10); // totale = align
        assert!(sep.ends_with("=====")); // gli ultimi 5 sono "="
        assert!(sep.starts_with("     ")); // i primi 5 sono spazi
    }

    #[test]
    fn test_make_separator_dollar() {
        let sep = make_separator('$', 3, 6);
        assert_eq!(sep, "   $$$");
    }

    #[test]
    fn test_make_separator_dash() {
        let sep = make_separator('-', 4, 8);
        assert_eq!(sep, "    ----");
    }

    #[test]
    fn test_make_separator_exact_align() {
        // align == width → nessuno spazio davanti
        let sep = make_separator('#', 6, 6);
        assert_eq!(sep, "######");
    }

    #[test]
    fn test_describe_office() {
        let (label, color) = describe_position("O");
        assert_eq!(label, "Office");
        assert_eq!(color, "\x1b[34m");
    }

    #[test]
    fn test_describe_remote() {
        let (label, color) = describe_position("R");
        assert_eq!(label, "Remote");
        assert_eq!(color, "\x1b[36m");
    }

    #[test]
    fn test_describe_client() {
        let (label, color) = describe_position("C");
        assert_eq!(label, "On-site (Client)");
        assert_eq!(color, "\x1b[33m");
    }

    #[test]
    fn test_describe_holiday() {
        let (label, color) = describe_position("H");
        assert_eq!(label, "Holiday");
        assert_eq!(color, "\x1b[45;97;1m");
    }

    #[test]
    fn test_describe_fallback() {
        let (label, color) = describe_position("X");
        assert_eq!(label, "X");
        assert_eq!(color, "\x1b[0m"); // default reset
    }
}
