//! Shared Core §15 timestamp lexical and calendar validation.

/// Returns whether `value` is the exact valid UTC nanosecond timestamp form.
///
/// The accepted form is `YYYY-MM-DDTHH:mm:ss.nnnnnnnnnZ`. Positive leap
/// seconds are accepted only on dates announced through 2016-12-31.
pub fn is_valid_utc_nanosecond_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 30
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'.'
        || bytes[29] != b'Z'
        || [0..4, 5..7, 8..10, 11..13, 14..16, 17..19, 20..29]
            .iter()
            .any(|range| !bytes[range.clone()].iter().all(u8::is_ascii_digit))
    {
        return false;
    }

    let year = decimal(&bytes[0..4]);
    let month = decimal(&bytes[5..7]);
    let day = decimal(&bytes[8..10]);
    let hour = decimal(&bytes[11..13]);
    let minute = decimal(&bytes[14..16]);
    let second = decimal(&bytes[17..19]);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    };
    let valid_leap_second = second == 60
        && hour == 23
        && minute == 59
        && day == days_in_month
        && is_announced_utc_leap_second(year, month, day);
    day != 0
        && day <= days_in_month
        && hour <= 23
        && minute <= 59
        && (second <= 59 || valid_leap_second)
}

fn decimal(digits: &[u8]) -> u32 {
    digits
        .iter()
        .fold(0, |value, digit| value * 10 + u32::from(digit - b'0'))
}

const fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Positive UTC leap-second dates announced through 2016-12-31.
const fn is_announced_utc_leap_second(year: u32, month: u32, day: u32) -> bool {
    matches!(
        (year, month, day),
        (
            1972 | 1981 | 1982 | 1983 | 1985 | 1992 | 1993 | 1994 | 1997 | 2012 | 2015,
            6,
            30
        ) | (
            1972 | 1973
                | 1974
                | 1975
                | 1976
                | 1977
                | 1978
                | 1979
                | 1987
                | 1989
                | 1990
                | 1995
                | 1998
                | 2005
                | 2008
                | 2016,
            12,
            31
        )
    )
}
