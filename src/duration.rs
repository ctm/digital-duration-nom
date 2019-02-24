use std::fmt;

pub const SECONDS_IN_MINUTE: u64 = 60;
const MINUTES_IN_HOUR: u64 = 60;
pub const SECONDS_IN_HOUR: u64 = MINUTES_IN_HOUR * SECONDS_IN_MINUTE;

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
        f.pad_integral(true, "", &result) // TODO: support negative durations
    }
}

impl Duration {
    pub fn new(secs: u64, nanos: u32) -> Self {
        Duration(std::time::Duration::new(secs, nanos))
    }

    pub const fn from_secs(secs: u64) -> Self {
        Self(std::time::Duration::from_secs(secs))
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

// TODO: probably want to remove this, so we have no dependencies on the
//       time crate.  OTOH, I'll wait until I've already gotten this crate
//       working with my other two projects.
//
//impl From<time::Duration> for Duration {
//    fn from(duration: time::Duration) -> Self {
//        let nanos = duration.num_nanoseconds().unwrap() % 1_000_000_000;
//
//        Self::new(duration.num_seconds() as u64, nanos as u32)
//    }
//}

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


#[cfg(test)]
mod tests {
    use super::Duration;

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
}
