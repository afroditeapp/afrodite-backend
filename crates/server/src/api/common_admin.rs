//! Common routes related to admin features










pub mod manager;
pub mod config;
pub mod perf;

pub use manager::*;
pub use self::config::*;
pub use perf::*;
