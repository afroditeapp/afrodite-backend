//! Common routes related to admin features

pub mod config;
pub mod maintenance;
pub mod manager;
pub mod notification;
pub mod report;
pub mod statistics;

pub use config::*;
pub use maintenance::*;
pub use manager::*;
pub use notification::*;
pub use report::*;
pub use statistics::*;
