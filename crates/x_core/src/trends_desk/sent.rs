//! Reply send counts for the desk HUD. "Today" is the calendar date in
//! America/Chicago, not the all-time total and not a UTC date.

use super::pack::ReplyMark;

/// Confirmed sends dated today in America/Chicago, and the all-time total.
/// A missing timestamp (`at_unix == 0`) is never counted as today.
pub fn sent_today_and_total(marks: &[ReplyMark], now_unix: i64) -> (usize, usize) {
    let today = chicago_ymd(now_unix);
    let mut today_count = 0;
    let mut total = 0;
    for mark in marks {
        if mark.status != "confirmed" {
            continue;
        }
        total += 1;
        if mark.at_unix > 0 && chicago_ymd(mark.at_unix) == today {
            today_count += 1;
        }
    }
    (today_count, total)
}

/// Civil date `YYYY-MM-DD` in America/Chicago.
pub fn chicago_ymd(unix: i64) -> String {
    crate::time::ymd(unix + i64::from(chicago_offset_secs(unix)))
}

/// UTC offset for America/Chicago at `unix`: −6 hours in CST, −5 in CDT.
/// DST starts at 02:00 local on the second Sunday in March and ends at
/// 02:00 local on the first Sunday in November (both 08:00 UTC).
pub fn chicago_offset_secs(unix: i64) -> i32 {
    let year: i32 = crate::time::ymd(unix)
        .get(..4)
        .and_then(|year| year.parse().ok())
        .unwrap_or(1970);
    let start = chicago_dst_start_utc(year);
    let end = chicago_dst_end_utc(year);
    if unix >= start && unix < end {
        -5 * 3600
    } else {
        -6 * 3600
    }
}

fn nth_sunday(year: i32, month: u32, n: u32) -> u32 {
    let first = format!("{year:04}-{month:02}-01T00:00:00Z");
    let unix = crate::time::parse_created_at(&first).unwrap_or(0);
    let (weekday, _) = crate::time::weekday_hour(unix);
    let first_sunday = if weekday == 0 {
        1
    } else {
        8 - u32::from(weekday)
    };
    first_sunday + 7 * n.saturating_sub(1)
}

fn chicago_dst_start_utc(year: i32) -> i64 {
    let day = nth_sunday(year, 3, 2);
    crate::time::parse_created_at(&format!("{year:04}-03-{day:02}T08:00:00Z")).unwrap_or(0)
}

fn chicago_dst_end_utc(year: i32) -> i64 {
    let day = nth_sunday(year, 11, 1);
    crate::time::parse_created_at(&format!("{year:04}-11-{day:02}T08:00:00Z")).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::super::pack::ReplyMark;
    use super::*;

    fn mark(at_unix: i64, status: &str) -> ReplyMark {
        ReplyMark {
            card_id: "1".into(),
            url: String::new(),
            at_unix,
            status: status.into(),
            text: String::new(),
            parent_id: String::new(),
            parent_url: String::new(),
            reply_id: String::new(),
            reply_url: String::new(),
        }
    }

    fn ts(raw: &str) -> i64 {
        crate::time::parse_created_at(raw).expect(raw)
    }

    #[test]
    fn chicago_offset_follows_us_dst_rules() {
        // 2026: DST starts March 8 02:00 CST / 08:00 UTC, ends Nov 1 02:00 CDT / 08:00 UTC.
        assert_eq!(chicago_offset_secs(ts("2026-03-08T07:59:59Z")), -6 * 3600);
        assert_eq!(chicago_offset_secs(ts("2026-03-08T08:00:00Z")), -5 * 3600);
        assert_eq!(chicago_offset_secs(ts("2026-11-01T07:59:59Z")), -5 * 3600);
        assert_eq!(chicago_offset_secs(ts("2026-11-01T08:00:00Z")), -6 * 3600);
        assert_eq!(chicago_offset_secs(ts("2026-11-24T20:00:00Z")), -6 * 3600);
        assert_eq!(chicago_ymd(ts("2026-11-24T06:00:00Z")), "2026-11-24");
        assert_eq!(chicago_ymd(ts("2026-11-24T05:59:59Z")), "2026-11-23");
    }

    #[test]
    fn today_uses_the_chicago_date_not_utc_and_not_the_lifetime_total() {
        // Midnight Nov 24, 2026 in Chicago is 06:00 UTC.
        let now = ts("2026-11-24T18:00:00Z"); // noon CST
        let start = ts("2026-11-24T06:00:00Z");
        let just_before = ts("2026-11-24T05:59:59Z"); // still Nov 23 in Chicago, already Nov 24 UTC
        let late_today = ts("2026-11-25T05:59:59Z"); // 23:59:59 CST Nov 24
        let next_day = ts("2026-11-25T06:00:00Z");
        let older = ts("2026-11-12T16:32:22Z");
        let marks = vec![
            mark(start, "confirmed"),
            mark(just_before, "confirmed"),
            mark(late_today, "confirmed"),
            mark(next_day, "confirmed"),
            mark(older, "confirmed"),
            mark(now, "pending"),
            mark(0, "confirmed"),
        ];
        let (today, total) = sent_today_and_total(&marks, now);
        assert_eq!(
            today, 2,
            "start of day and late evening Chicago count; UTC-tomorrow-but-Chicago-today too"
        );
        assert_eq!(
            total, 6,
            "all confirmed marks, including no-timestamp and other days"
        );
        // Just before midnight Chicago, the UTC date is already the next day.
        assert_eq!(chicago_ymd(just_before), "2026-11-23");
        assert_eq!(chicago_ymd(late_today), "2026-11-24");
        assert_eq!(crate::time::ymd(just_before), "2026-11-24");
        assert_eq!(crate::time::ymd(late_today), "2026-11-25");
    }

    #[test]
    fn a_zero_timestamp_is_never_today() {
        let now = ts("2026-11-24T18:00:00Z");
        let (today, total) = sent_today_and_total(&[mark(0, "confirmed")], now);
        assert_eq!(today, 0);
        assert_eq!(total, 1);
    }
}
