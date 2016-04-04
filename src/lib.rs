//! Experimental extensions to the `std::time` module.
#![warn(missing_docs)]

pub use duration::DurationExt;
#[cfg(feature = "beta")]
pub use system_time::SystemTimeExt;

mod duration;
#[cfg(feature = "beta")]
mod system_time;
