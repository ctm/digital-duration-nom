//! # digital-duration-nom
//!
//! digital-duration-nom provides a `Duration` type that wraps
//! `std::time::Duration` to provide for parsing "digital" (e.g.,
//! `1:23:45`), as opposed to "human readable" (e.g., `One hour,
//! twenty three minutes, forty five seconds`) string representations.
//! It also implements `std::fmt::Display` and provides a trait so
//! that `Option<Duration>` can be printed.
//!
//! ## Example
//!
//! TODO
//!
//! ### Caveat Emptor
//!
//! I created this crate purely for my own use, but I've decided to
//! make more of my private repositories public, so here it is.

pub mod duration;
pub mod option_display;
