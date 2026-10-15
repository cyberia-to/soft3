//! Integer civil time: `YYYY-MM-DD HH:MM:SS` ↔ unix seconds, no floats, no
//! locale. Proleptic Gregorian, UTC only (block times are UTC).

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
pub const fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 } as u64;
    let doy = (153 * mp + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

pub const fn civil_to_unix(y: i64, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> u64 {
    let days = days_from_civil(y, m, d);
    (days * 86_400 + hh as i64 * 3_600 + mm as i64 * 60 + ss as i64) as u64
}

/// Parse `YYYY-MM-DD HH:MM:SS` (what the burial extractors wrote; a `T`
/// separator and a trailing `Z` are tolerated) into unix seconds.
pub fn parse_timestamp(s: &str) -> Option<u64> {
    let s = s.trim().trim_end_matches('Z');
    if s.len() < 19 {
        return None;
    }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' || (b[10] != b' ' && b[10] != b'T') || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let num = |a: usize, z: usize| s[a..z].parse::<u32>().ok();
    let y = s[0..4].parse::<i64>().ok()?;
    let m = num(5, 7)?;
    let d = num(8, 10)?;
    let hh = num(11, 13)?;
    let mm = num(14, 16)?;
    let ss = num(17, 19)?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return None;
    }
    Some(civil_to_unix(y, m, d, hh, mm, ss))
}

/// Format unix seconds as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn format_unix(t: u64) -> String {
    let days = (t / 86_400) as i64;
    let rem = t % 86_400;
    // civil_from_days (Hinnant)
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        (rem % 3_600) / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_and_known_dates() {
        assert_eq!(civil_to_unix(1970, 1, 1, 0, 0, 0), 0);
        assert_eq!(civil_to_unix(2000, 3, 1, 0, 0, 0), 951_868_800);
        assert_eq!(parse_timestamp("2022-10-03 20:28:53"), Some(1_664_828_933));
        assert_eq!(parse_timestamp("2026-06-14T08:36:41Z"), Some(1_781_426_201));
        assert_eq!(super::super::LAUNCH_UNIX, 1_793_836_800);
    }

    #[test]
    fn round_trip() {
        for t in [0u64, 951_868_800, 1_664_828_933, 1_793_836_800, 4_102_444_799] {
            let s = format_unix(t);
            assert_eq!(parse_timestamp(&s), Some(t), "{s}");
        }
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_timestamp(""), None);
        assert_eq!(parse_timestamp("2022-13-03 20:28:53"), None);
        assert_eq!(parse_timestamp("2022/10/03 20:28:53"), None);
    }
}
