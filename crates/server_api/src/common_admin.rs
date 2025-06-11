//! Common routes related to admin features

pub mod config;
pub mod manager;
pub mod statistics;
pub mod report;
pub mod maintenance;
pub mod notification;

pub use manager::*;
pub use statistics::*;
pub use config::*;
pub use report::*;
pub use maintenance::*;
pub use notification::*;
