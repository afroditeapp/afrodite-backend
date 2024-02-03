#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Data types for API and database.

pub mod perf;
pub use perf::*;

pub mod time;
pub use time::*;

mod markers;

mod macros;
