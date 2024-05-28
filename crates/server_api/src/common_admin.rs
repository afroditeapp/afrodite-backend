//! Common routes related to admin features

pub mod config;
pub mod manager;
pub mod perf;

pub use manager::*;
pub use perf::*;

pub use self::config::*;
