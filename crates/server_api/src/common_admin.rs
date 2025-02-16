//! Common routes related to admin features

pub mod config;
pub mod manager;
pub mod perf;
pub mod report;

pub use manager::*;
pub use perf::*;
pub use config::*;
pub use report::*;
