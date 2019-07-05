/// This is the best way I (a Rust newbie) currently know to make it
/// so that an Option<T: Display> can be easily used in println!.
///
/// FWIW, This doesn't have to do with "sports metrics", per-se, but
/// that name itself is a misnomer.  This functionality is, however,
/// fairly useful when creating fixed-width output where a particular
/// field value is optional and that use-case comes up in the same
/// place where we want to parse and format Durations.
use std::fmt::{self, Display, Formatter};

pub trait OptionDisplay<T: Display> {
    fn option(&self) -> &Option<T>;
}

impl<T: Display> OptionDisplay<T> for Option<T> {
    fn option(&self) -> &Option<T> {
        self
    }
}

impl<T: Display> Display for &dyn OptionDisplay<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.option() {
            None => " ".fmt(f),
            Some(value) => value.fmt(f),
        }
    }
}
