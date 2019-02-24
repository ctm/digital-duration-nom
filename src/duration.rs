use std::fmt;
use nom::digit;
use nom::types::CompleteStr;
use std::str::FromStr;

const SECONDS_IN_MINUTE: u64 = 60;
const MINUTES_IN_HOUR: u64 = 60;
const SECONDS_IN_HOUR: u64 = MINUTES_IN_HOUR * SECONDS_IN_MINUTE;
const NANOSECONDS_IN_SECOND: u32 = 1_000_000_000;
const TENTHS_IN_NANOSECOND: u32 = NANOSECONDS_IN_SECOND / 10;

custom_derive! {
    // Can't use NewtypeSum w/o unstable
    #[derive(Copy, Clone, Debug, PartialEq, NewtypeAdd, NewtypeDiv(u32), NewtypeDeref, Ord, Eq, PartialOrd)]
    pub struct Duration(std::time::Duration);
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let all_secs = self.as_secs();
        let hours = all_secs / SECONDS_IN_HOUR;
        let minutes = all_secs / SECONDS_IN_MINUTE % SECONDS_IN_MINUTE;
        let seconds = all_secs % SECONDS_IN_MINUTE;
        let tenths = self.subsec_millis() / 100 as u32;

        let precision = match f.precision() {
            Some(p) => p,
            None => 0,
        };

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

impl From<f64> for Duration {
    fn from(f64: f64) -> Self {
        Self::new(f64.trunc() as u64, (f64.fract() * 1e9) as u32)
    }
}

impl<'a> std::iter::Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        Duration(iter.map(|d| d.0).sum())
    }
}

impl Into<f64> for Duration {
    fn into(self) -> f64 {
        self.as_secs() as f64 + f64::from(self.subsec_nanos()) * 1e-9
    }
}

impl Into<std::time::Duration> for Duration {
    fn into(self) -> std::time::Duration {
        self.0
    }
}

impl std::ops::Mul for Duration {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::from(Into::<f64>::into(self) * Into::<f64>::into(rhs))
    }
}

impl std::ops::AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

// This is the best way I (a Rust newbie) currently know to make it so that
// an Option<Duration> can be easily used in println!.  Logically, since this
// works with Option<T>, it really doesn't belong in duration.rs, but I haven't
// figured out where I wnt it.
pub trait Printable<T>
where T: fmt::Display {
    fn option(&self) -> &Option<T>;
}

impl<T> Printable<T> for Option<T>
where T: fmt::Display {
    fn option(&self) -> &Option<T> {
        self
    }
}

impl fmt::Display for &dyn Printable<Duration> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.option() {
            None => " ".fmt(f),
            Some(value) => value.fmt(f),
        }
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

named!(hour_prefix<CompleteStr, Duration>,
  do_parse!(
    digits: digit >>
    tag!(":") >>
    (Duration::new(digits.parse::<u64>().unwrap() * SECONDS_IN_HOUR, 0))
  )
);

named!(zero_through_five<CompleteStr, u8>,
  do_parse!(
    digit: one_of!("012345") >>
    (digit as u8 - b'0')
  )
);

named!(single_digit<CompleteStr, u8>,
  do_parse!(
    digit: one_of!("0123456789") >>
    (digit as u8 - b'0')
  )
);

named!(double_digit_minute_prefix<CompleteStr, Duration>,
  do_parse!(
    tens: zero_through_five >>
    ones: single_digit >>
    tag!(":") >>
    (Duration::new(((u64::from(tens) * 10) + u64::from(ones)) * SECONDS_IN_MINUTE, 0))
  )
);

named!(single_digit_minute_prefix<CompleteStr, Duration>,
  do_parse!(
    ones: single_digit >>
    tag!(":") >>
    (Duration::new(u64::from(ones) * SECONDS_IN_MINUTE, 0))
  )
);

named!(double_digit_seconds<CompleteStr, Duration>,
  do_parse!(
    tens: zero_through_five >>
    ones: single_digit >>
    (Duration::new((u64::from(tens) * 10) + u64::from(ones), 0))
  )
);

named!(single_digit_seconds<CompleteStr, Duration>,
  do_parse!(
    ones: single_digit >>
    (Duration::new(u64::from(ones), 0))
  )
);

named!(tenths<CompleteStr, Duration>,
  do_parse!(
    tag!(".") >>
    tenth: single_digit >>
    (Duration::new(0, u32::from(tenth) * TENTHS_IN_NANOSECOND))
  )
);

named!(hour_and_minute_prefix<CompleteStr, Duration>,
  alt!(
    do_parse!(
      hours: hour_prefix >>
      minutes: double_digit_minute_prefix >>
      (hours + minutes)
    ) |
    double_digit_minute_prefix
  )
);

named!(minute_prefix<CompleteStr, Duration>,
  alt!(hour_and_minute_prefix | single_digit_minute_prefix)
);

named!(prefix_and_double_digit_seconds<CompleteStr, Duration>,
  do_parse!(
    minutes: opt!(minute_prefix) >>
    seconds: double_digit_seconds >>
    (match minutes {
      None => seconds,
      Some(minutes) => minutes + seconds
    })
  )
);

named!(without_decimal<CompleteStr, Duration>,
  alt!(prefix_and_double_digit_seconds | single_digit_seconds)
);

named!(pub duration_parser<CompleteStr, Duration>,
  do_parse!(
    seconds: without_decimal >>
    tenths: opt!(tenths) >>
    (match tenths {
      None => seconds,
      Some(tenths) => seconds + tenths,
    })
  )
);

#[derive(Debug)]
pub enum ParseDurationErr {
    Unspecified, // TODO
    LeftoverCharacters,
}

impl FromStr for Duration {
    type Err = ParseDurationErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match duration_parser(CompleteStr(s)) {
            Ok((remaining, duration)) => {
                if remaining.is_empty() {
                    Ok(duration)
                } else {
                    Err(ParseDurationErr::LeftoverCharacters)
                }
            }
            // TODO would be nice to use _e here
            Err(_e) => Err(ParseDurationErr::Unspecified),
        }
    }
}

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
        assert_eq!(
            Duration::new(3600, 0),
            hour_prefix(CompleteStr("1:")).unwrap().1
        );
        assert_eq!(
            Duration::new(36000, 0),
            hour_prefix(CompleteStr("10:")).unwrap().1
        );
    }

    #[test]
    fn test_double_digit_minute_prefix() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE, 0),
            double_digit_minute_prefix(CompleteStr("11:06")).unwrap().1
        );
    }

    #[test]
    fn test_tenths() {
        assert_eq!(
            Duration::new(0, 900_000_000),
            tenths(CompleteStr(".9")).unwrap().1
        );
        assert_eq!(
            Duration::new(1, 0),
            tenths(CompleteStr(".9")).unwrap().1 + tenths(CompleteStr(".1")).unwrap().1
        );
    }

    #[test]
    fn test_hour_and_minute_prefix() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE, 0),
            hour_and_minute_prefix(CompleteStr("11:06")).unwrap().1
        );
    }

    #[test]
    fn test_minute_prefix() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE, 0),
            minute_prefix(CompleteStr("11:06")).unwrap().1
        );
    }

    #[test]
    fn test_prefix_and_double_digit_seconds() {
        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE + 6, 0),
            prefix_and_double_digit_seconds(CompleteStr("11:06"))
                .unwrap()
                .1
        );
    }

    #[test]
    fn test_duration_parser() {
        assert_eq!(
            Duration::new(8 * SECONDS_IN_MINUTE + 22, 0),
            duration_parser(CompleteStr("8:22")).unwrap().1
        );

        assert_eq!(
            Duration::new(1 * SECONDS_IN_MINUTE + 15, 3 * TENTHS_IN_NANOSECOND),
            duration_parser(CompleteStr("1:15.3")).unwrap().1
        );

        assert_eq!(
            Duration::new(2 * SECONDS_IN_HOUR + 25 * SECONDS_IN_MINUTE + 36, 0),
            duration_parser(CompleteStr("2:25:36")).unwrap().1
        );

        assert_eq!(
            Duration::new(
                2 * SECONDS_IN_HOUR + 25 * SECONDS_IN_MINUTE + 36,
                7 * TENTHS_IN_NANOSECOND
            ),
            duration_parser(CompleteStr("2:25:36.7")).unwrap().1
        );

        assert_eq!(
            Duration::new(20 * SECONDS_IN_MINUTE + 29, 8 * TENTHS_IN_NANOSECOND),
            duration_parser(CompleteStr("20:29.8")).unwrap().1
        );

        assert_eq!(
            Duration::new(11 * SECONDS_IN_MINUTE + 6, 0),
            duration_parser(CompleteStr("11:06")).unwrap().1
        );

        assert_eq!(Duration::new(0, 0), duration_parser(CompleteStr("0")).unwrap().1);

        assert_eq!(Duration::new(1, 0), duration_parser(CompleteStr("1")).unwrap().1);

        assert_eq!(Duration::new(5, 0), duration_parser(CompleteStr("05")).unwrap().1);

        assert_eq!(Duration::new(10, 0), duration_parser(CompleteStr("10")).unwrap().1);

        assert_eq!(
            Duration::new(8 * SECONDS_IN_MINUTE + 1, 6 * TENTHS_IN_NANOSECOND),
            duration_parser(CompleteStr("8:01.6")).unwrap().1
        );
    }
}
