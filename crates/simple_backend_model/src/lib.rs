#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Data types for API and database.

pub use simple_backend_utils::{
    diesel_bytes_wrapper, diesel_db_i16_is_i8_struct, diesel_db_i16_is_u8_struct,
    diesel_i16_wrapper, diesel_i32_wrapper, diesel_i64_wrapper, diesel_non_empty_string_wrapper,
    diesel_string_wrapper, diesel_uuid_wrapper, string::NonEmptyString,
};
pub use simple_diesel_enum_macro::SimpleDieselEnum;

pub mod perf;
pub use perf::*;

pub mod time;
pub use time::*;

pub mod ip;
pub use ip::*;

pub mod version;
pub use version::*;

pub mod image_processing;
pub use image_processing::*;

#[cfg(test)]
mod tests {
    // Ignore unused dependency warning. Unit tests need this dependency.
    use uuid as _;
}
