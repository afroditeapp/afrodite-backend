//! Config given as command line arguments

use std::path::PathBuf;

use clap::{arg, Args};

#[derive(Args, Debug, Clone)]
pub struct ServerModeArgs {
    /// Set data directory. Overrides config file value.
    #[arg(short, long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// Use in RAM mode for SQLite.
    #[arg(short, long)]
    pub sqlite_in_ram: bool,
}
