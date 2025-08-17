//! Date parsing utilities for Tooka.
//!
//! Supports both absolute dates (RFC3339 format) and relative dates
//! like "now", "-7d", "+2w", etc.

use chrono::{DateTime, Duration, Utc};
use std::str::FromStr;

/// Parses a date string that can be either:
/// - RFC3339 format (e.g., "2025-06-20T00:00:00Z")
/// - ISO 8601 date format (e.g., "2025-06-20")
/// - Relative format (e.g., "now", "-7d", "+2w", "-1m", "+3y")
pub fn parse_date(date_str: &str) -> Result<DateTime<Utc>, String> {
    let date_str = date_str.trim();

    // Handle "now" keyword
    if date_str.eq_ignore_ascii_case("now") {
        return Ok(Utc::now());
    }

    // Try to parse as RFC3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try to parse as ISO 8601 date (YYYY-MM-DD)
    if let Ok(naive_date) = chrono::NaiveDate::from_str(date_str) {
        if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
            return Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc));
        }
    }

    // Try to parse as relative date
    if let Ok(dt) = parse_relative_date(date_str) {
        return Ok(dt);
    }

    Err(format!(
        "Invalid date format: '{date_str}'. Expected RFC3339, ISO 8601 (YYYY-MM-DD), or relative format (e.g., 'now', '-7d', '+2w')"
    ))
}

/// Parses relative date formats like "-7d", "+2w", "-1m", "+3y"
fn parse_relative_date(date_str: &str) -> Result<DateTime<Utc>, String> {
    let date_str = date_str.trim();

    // Must start with + or - for relative dates
    let (is_negative, date_str) = match date_str.chars().next() {
        Some('+') => (false, &date_str[1..]),
        Some('-') => (true, &date_str[1..]),
        _ => return Err("Relative dates must start with + or -".to_string()),
    };

    if date_str.is_empty() {
        return Err("Empty relative date".to_string());
    }

    // Parse the number and unit
    let unit_char = date_str.chars().last().unwrap();
    let number_str = &date_str[..date_str.len() - 1];

    let number: i64 = number_str
        .parse()
        .map_err(|_| format!("Invalid number in relative date: '{number_str}'"))?;

    let number = if is_negative { -number } else { number };

    let duration = match unit_char.to_ascii_lowercase() {
        'd' => Duration::days(number),
        'w' => Duration::weeks(number),
        'm' => Duration::days(number * 30), // Approximate month as 30 days
        'y' => Duration::days(number * 365), // Approximate year as 365 days
        'h' => Duration::hours(number),
        's' => Duration::seconds(number),
        _ => {
            return Err(format!(
                "Invalid time unit '{unit_char}'. Supported units: d (days), w (weeks), m (months), y (years), h (hours), s (seconds)"
            ));
        }
    };

    Ok(Utc::now() + duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_rfc3339() {
        let result = parse_date("2025-06-20T12:00:00Z");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 20);
    }

    #[test]
    fn test_parse_iso_date() {
        let result = parse_date("2025-06-20");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 20);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_parse_now() {
        let result = parse_date("now");
        assert!(result.is_ok());

        let result = parse_date("NOW");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_relative_days() {
        let result = parse_date("-7d");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let expected = Utc::now() - Duration::days(7);
        // Allow for small time differences due to test execution time
        assert!((dt - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_relative_weeks() {
        let result = parse_date("+2w");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let expected = Utc::now() + Duration::weeks(2);
        assert!((dt - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_relative_months() {
        let result = parse_date("-1m");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let expected = Utc::now() - Duration::days(30);
        assert!((dt - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_relative_years() {
        let result = parse_date("+1y");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let expected = Utc::now() + Duration::days(365);
        assert!((dt - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_invalid_formats() {
        assert!(parse_date("invalid").is_err());
        assert!(parse_date("7d").is_err()); // Missing sign
        assert!(parse_date("-7x").is_err()); // Invalid unit
        assert!(parse_date("-").is_err()); // Missing number and unit
        assert!(parse_date("-abc").is_err()); // Invalid number
    }
}
