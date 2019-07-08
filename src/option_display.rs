/// This is the best way I (a Rust newbie) currently know to make it
/// so that an Option<T: Display> can be easily used in println!.
///
/// FWIW, Although this code isn't specific "digital durations", it is
/// handy when creating fixed-width output that includes digital
/// durations, so it's included here.
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
