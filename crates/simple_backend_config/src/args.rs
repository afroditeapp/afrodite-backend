//! Config given as command line arguments

use std::path::PathBuf;

use clap::Args;

const DEFAULT_DATA_DIR_NAME: &str = "data";

#[derive(Args, Debug, Clone)]
pub struct ServerMode {
    /// Set data directory for SQLite databases and other files.
    #[arg(short, long, value_name = "DIR", default_value = DEFAULT_DATA_DIR_NAME)]
    pub data_dir: PathBuf,

    /// Use in RAM mode for SQLite.
    #[arg(short, long)]
    pub sqlite_in_ram: bool,
}

impl ServerMode {
    pub fn new_with_default_data_dir(sqlite_in_ram: bool) -> Self {
        Self {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR_NAME),
            sqlite_in_ram,
        }
    }
}

impl Default for ServerMode {
    fn default() -> Self {
        Self::new_with_default_data_dir(false)
    }
}
