use chrono::{DateTime, Utc};

pub fn now_iso8601() -> String {
    Utc::now().to_rfc3339()
}

pub fn now_timestamp_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn parse_date(date_str: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(date_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
