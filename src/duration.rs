use derive_more::{Add, AddAssign, Deref, Div, Into};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, one_of},
    combinator::{map, opt},
    sequence::{preceded, terminated, tuple},
    IResult,
};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::ops::Mul;
use std::str::FromStr;

const SECONDS_IN_MINUTE: u64 = 60;
const MINUTES_IN_HOUR: u64 = 60;
const SECONDS_IN_HOUR: u64 = MINUTES_IN_HOUR * SECONDS_IN_MINUTE;
const NANOSECONDS_IN_SECOND: u32 = 1_000_000_000;
const TENTHS_IN_NANOSECOND: u32 = NANOSECONDS_IN_SECOND / 10;

#[derive(Add, AddAssign, Clone, Copy, Debug, Deref, Div, Eq, Into, Ord, PartialEq, PartialOrd)]
pub struct Duration(std::time::Duration);

impl Duration {
    pub fn new(secs: u64, nanos: u32) -> Self {
        Duration(std::time::Duration::new(secs, nanos))
    }

    pub const fn from_secs(secs: u64) -> Self {
        Self(std::time::Duration::from_secs(secs))
    }

    pub fn new_hour_min_sec(hours: u64, mins: u8, secs: u8) -> Self {
        Self::new_min_sec(hours * MINUTES_IN_HOUR + u64::from(mins), secs)
    }

    pub fn new_min_sec(mins: u64, secs: u8) -> Self {
        Self::new_min_sec_tenths(mins, secs, 0)
    }

    pub fn new_min_sec_tenths(mins: u64, secs: u8, tenths: u8) -> Self {
        Self::new(
            mins * SECONDS_IN_MINUTE + u64::from(secs),
            u32::from(tenths) * 100_000_000,
        )
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let all_secs = self.as_secs();
        let hours = all_secs / SECONDS_IN_HOUR;
        let minutes = all_secs / SECONDS_IN_MINUTE % SECONDS_IN_MINUTE;
        let seconds = all_secs % SECONDS_IN_MINUTE;
        let tenths = self.subsec_millis() / 100u32;

        let precision = f.precision().unwrap_or(0);
        let mut result = String::new();

        if hours > 0 {
            result.push_str(&format!("{}:{:02}:{:02}", hours, minutes, seconds));
        } else if minutes > 0 {
            result.push_str(&format!("{}:{:02}", minutes, seconds));
        } else {
            result.push_str(&seconds.to_string());
        }

        if tenths > 0 || precision > 0 {
            result.push_str(&format!(".{}", tenths));
        }
        f.pad_integral(true, "", &result)
    }
}

impl From<f64> for Duration {
    fn from(f64: f64) -> Self {
        Self::new(f64.trunc() as u64, (f64.fract() * 1e9) as u32)
    }
}

impl FromStr for Duration {
    type Err = ParseDurationErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match duration_parser(s) {
            Ok((remaining, duration)) => {
                if remaining.is_empty() {
                    Ok(duration)
                } else {
                    Err(ParseDurationErr::LeftoverCharacters)
                }
            }
            // We're throwing away the Err that nom returns, so that
            // our FromStr implementation doesn't have to depend on nom,
            // even though it currently does.
            Err(_) => Err(ParseDurationErr::Unspecified),
        }
    }
}

impl Into<f64> for Duration {
    fn into(self) -> f64 {
        self.as_secs() as f64 + f64::from(self.subsec_nanos()) * 1e-9
    }
}

impl Mul for Duration {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::from(Into::<f64>::into(self) * Into::<f64>::into(rhs))
    }
}

impl<'a> std::iter::Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        Duration(iter.map(|d| d.0).sum())
    }
}

//    8:22
//    1:15.0
// 2:25:36
//   20:29.8
//   11:06
//       0
//       1
//      05
//      10

// Here's some ruby regexp code that shows what I'm going for

// HOUR_PREFIX = /([0-9]+):/
// DOUBLE_DIGIT_MINUTE_PREFIX = /([0-5][0-9]):/
// SINGLE_DIGIT_MINUTE_PREFIX = /([0-9]):/
// DOUBLE_DIGIT_SECONDS = /([0-5][0-9])/
// SINGLE_DIGIT_SECONDS = /([0-9])/
// TENTHS = /\.([0-9])/
// HOUR_AND_MINUTE_PREFIX = /#{HOUR_PREFIX}?#{DOUBLE_DIGIT_MINUTE_PREFIX}/
// MINUTE_PREFIX = /#{HOUR_AND_MINUTE_PREFIX}|#{SINGLE_DIGIT_MINUTE_PREFIX}/
// PREFIX_AND_DOUBLE_DIGIT_SECONDS = /#{MINUTE_PREFIX}?#{DOUBLE_DIGIT_SECONDS}/
// WITHOUT_DECIMAL = /#{PREFIX_AND_DOUBLE_DIGIT_SECONDS}|#{SINGLE_DIGIT_SECONDS}/
// ALL = /#{WITHOUT_DECIMAL}#{TENTHS}?/

fn hour_prefix(input: &str) -> IResult<&str, Duration> {
    map(terminated(digit1, tag(":")), |digits: &str| {
        Duration::new(digits.parse::<u64>().unwrap() * SECONDS_IN_HOUR, 0)
    })(input)
}

fn zero_through_five(input: &str) -> IResult<&str, u8> {
    map(one_of("012345"), |digit| digit as u8 - b'0')(input)
}

fn single_digit(input: &str) -> IResult<&str, u8> {
    map(one_of("0123456789"), |digit| digit as u8 - b'0')(input)
}

fn double_digit_minute_prefix(input: &str) -> IResult<&str, Duration> {
    map(
        tuple((zero_through_five, terminated(single_digit, tag(":")))),
        |(tens, ones)| {
            Duration::new(
                ((u64::from(tens) * 10) + u64::from(ones)) * SECONDS_IN_MINUTE,
                0,
            )
        },
    )(input)
}

fn single_digit_minute_prefix(input: &str) -> IResult<&str, Duration> {
    map(terminated(single_digit, tag(":")), |ones| {
        Duration::new(u64::from(ones) * SECONDS_IN_MINUTE, 0)
    })(input)
}

fn double_digit_seconds(input: &str) -> IResult<&str, Duration> {
    map(tuple((zero_through_five, single_digit)), |(tens, ones)| {
        Duration::new((u64::from(tens) * 10) + u64::from(ones), 0)
    })(input)
}

fn single_digit_seconds(input: &str) -> IResult<&str, Duration> {
    map(single_digit, |ones| Duration::new(u64::from(ones), 0))(input)
}

fn tenths(input: &str) -> IResult<&str, Duration> {
    map(preceded(tag("."), single_digit), |tenth| {
        Duration::new(0, u32::from(tenth) * TENTHS_IN_NANOSECOND)
    })(input)
}

fn hours_and_double_digit_minute_prefix(input: &str) -> IResult<&str, Duration> {
    map(
        tuple((hour_prefix, double_digit_minute_prefix)),
        |(hours, minutes)| hours + minutes,
    )(input)
}

fn hour_and_minute_prefix(input: &str) -> IResult<&str, Duration> {
    alt((
        hours_and_double_digit_minute_prefix,
        double_digit_minute_prefix,
    ))(input)
}

fn minute_prefix(input: &str) -> IResult<&str, Duration> {
    alt((hour_and_minute_prefix, single_digit_minute_prefix))(input)
}

fn prefix_and_double_digit_seconds(input: &str) -> IResult<&str, Duration> {
    map(
        tuple((opt(minute_prefix), double_digit_seconds)),
        |(minutes, seconds)| match minutes {
            None => seconds,
            Some(minutes) => minutes + seconds,
        },
    )(input)
}

fn without_decimal(input: &str) -> IResult<&str, Duration> {
    alt((prefix_and_double_digit_seconds, single_digit_seconds))(input)
}

pub fn duration_parser(input: &str) -> IResult<&str, Duration> {
    map(
        tuple((without_decimal, opt(tenths))),
        |(seconds, tenths)| match tenths {
            None => seconds,
            Some(tenths) => seconds + tenths,
        },
    )(input)
}

#[derive(Debug)]
pub enum ParseDurationErr {
    Unspecified,
    LeftoverCharacters,
}

impl Display for ParseDurationErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ParseDurationErr::*;
        match self {
            Unspecified => write!(f, "Unspecified error"),
            LeftoverCharacters => write!(f, "Leftover characters"),
        }
    }
}

impl Error for ParseDurationErr {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{:<7}", Duration::new(35, 0)), "35     ");
        assert_eq!(format!("{:>7}", Duration::new(35, 0)), "     35");
        assert_eq!(format!("{:7}", Duration::new(35, 0)), "     35");
        assert_eq!(format!("{:7}", Duration::new_min_sec(49, 32)), "  49:32");
        assert_eq!(
            format!("{:7.1}", Duration::new_min_sec_tenths(9, 12, 3)),
            " 9:12.3"
        );
    }

    #[test]
    fn test_hour_prefix() {
        assert_eq!(Duration::new(3600, 0), hour_prefix("1:").unwrap().1);
        assert_eq!(Duration::new(36000, 0), hour_prefix("10:").unwrap().1);
    }

    #[test]
    fn test_double_digit_minute_prefix() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE, 0),
            double_digit_minute_prefix("11:06").unwrap().1
        );
    }

    #[test]
    fn test_tenths() {
        assert_eq!(Duration::new(0, 900_000_000), tenths(".9").unwrap().1);
        assert_eq!(
            Duration::new(1, 0),
            tenths(".9").unwrap().1 + tenths(".1").unwrap().1
        );
    }

    #[test]
    fn test_hour_and_minute_prefix() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE, 0),
            hour_and_minute_prefix("11:06").unwrap().1
        );
    }

    #[test]
    fn test_minute_prefix() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE, 0),
            minute_prefix("11:06").unwrap().1
        );
    }

    #[test]
    fn test_prefix_and_double_digit_seconds() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE + 6, 0),
            prefix_and_double_digit_seconds("11:06").unwrap().1
        );
    }

    #[test]
    fn test_duration_parser() {
        assert_eq!(
            Duration::new(8 * SECONDS_IN_MINUTE + 22, 0),
            duration_parser("8:22").unwrap().1
        );

        assert_eq!(
            Duration::new(1 * SECONDS_IN_MINUTE + 15, 3 * TENTHS_IN_NANOSECOND),
            duration_parser("1:15.3").unwrap().1
        );

        assert_eq!(
            Duration::new(2 * SECONDS_IN_HOUR + 25 * SECONDS_IN_MINUTE + 36, 0),
            duration_parser("2:25:36").unwrap().1
        );

        assert_eq!(
            Duration::new(
                2 * SECONDS_IN_HOUR + 25 * SECONDS_IN_MINUTE + 36,
                7 * TENTHS_IN_NANOSECOND
            ),
            duration_parser("2:25:36.7").unwrap().1
        );

        assert_eq!(
            Duration::new(20 * SECONDS_IN_MINUTE + 29, 8 * TENTHS_IN_NANOSECOND),
            duration_parser("20:29.8").unwrap().1
        );

        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE + 6, 0),
            duration_parser("11:06").unwrap().1
        );

        assert_eq!(Duration::new(0, 0), duration_parser("0").unwrap().1);

        assert_eq!(Duration::new(1, 0), duration_parser("1").unwrap().1);

        assert_eq!(Duration::new(5, 0), duration_parser("05").unwrap().1);

        assert_eq!(Duration::new(10, 0), duration_parser("10").unwrap().1);

        assert_eq!(
            Duration::new(8 * SECONDS_IN_MINUTE + 1, 6 * TENTHS_IN_NANOSECOND),
            duration_parser("8:01.6").unwrap().1
        );
    }
}
