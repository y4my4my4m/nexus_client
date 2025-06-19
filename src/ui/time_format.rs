use chrono::{DateTime, Local, Duration, Datelike};
use chrono_humanize::HumanTime;
use chrono::TimeZone;

/// Format a message timestamp for chat display, Discord-style.
/// - <5min: humanized ("just now", "2 minutes ago")
/// - Today: "9:39 PM"
/// - Yesterday: "Yesterday, 9:39 PM"
/// - Older: "6/16/25, 8:30 AM"
pub fn format_message_timestamp(ts: i64, now: DateTime<Local>) -> String {
    let dt = Local.timestamp_opt(ts, 0).single();
    if let Some(dt) = dt {
        let duration = now.signed_duration_since(dt);
        if duration < Duration::minutes(5) {
            HumanTime::from(dt).to_string()
        } else if dt.date_naive() == now.date_naive() {
            dt.format("%-I:%M %p").to_string()
        } else if dt.date_naive() == (now - Duration::days(1)).date_naive() {
            format!("Yesterday, {}", dt.format("%-I:%M %p"))
        } else {
            dt.format("%-m/%-d/%y, %-I:%M %p").to_string()
        }
    } else {
        "?".to_string()
    }
}

/// Format a date for a date delimiter (e.g., "June 16th, 2025")
pub fn format_date_delimiter(ts: i64) -> String {
    let dt = Local.timestamp_opt(ts, 0).single();
    if let Some(dt) = dt {
        let day = dt.day();
        let suffix = match day {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            _ => "th",
        };
        format!("{} {}{}, {}", dt.format("%B"), day, suffix, dt.year())
    } else {
        "?".to_string()
    }
}
