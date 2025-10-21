#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Data types for API and database.

pub use simple_diesel_enum_macro::SimpleDieselEnum;

pub mod perf;
pub use perf::*;

pub mod time;
pub use time::*;

pub mod ip;
pub use ip::*;

pub mod version;
pub use version::*;

pub mod string;
pub use string::*;

mod macros;

#[cfg(test)]
mod tests {
    // Ignore unused dependency warning. Unit tests need this dependency.
    use uuid as _;
}
