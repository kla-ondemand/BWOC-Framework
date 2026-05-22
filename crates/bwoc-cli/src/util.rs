//! Cross-module helpers for `bwoc-cli`. Promote to `bwoc-core` if a
//! second crate ever needs them.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Best-effort UTC ISO 8601 timestamp without pulling in a date crate.
/// Falls back to shelling out to `date -u` if `SystemTime` arithmetic fails.
pub fn utc_now_iso8601() -> String {
    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs() as i64;
        return format_iso8601(secs);
    }
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Convert a POSIX timestamp (seconds since UNIX epoch) to UTC ISO 8601.
/// Implements the proleptic Gregorian calendar conversion directly to avoid
/// pulling in `chrono` or `time` crates.
pub fn format_iso8601(mut secs: i64) -> String {
    let day_secs = 86400i64;
    let days = secs.div_euclid(day_secs);
    secs = secs.rem_euclid(day_secs);
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;

    // Days since 1970-01-01 → (year, month, day)
    // Algorithm: shift epoch to 0000-03-01 and use Howard Hinnant's date math.
    let z = days + 719_468; // days since 0000-03-01
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 {
        (mp + 3) as u8
    } else {
        (mp - 9) as u8
    };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{mins:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_format_anchors() {
        assert_eq!(format_iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_iso8601(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(format_iso8601(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(format_iso8601(1_709_164_800), "2024-02-29T00:00:00Z");
    }
}
