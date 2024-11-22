//! Config given as command line arguments

use std::path::PathBuf;

use clap::{arg, Args, ValueEnum};

#[derive(Args, Debug, Clone)]
pub struct ServerModeArgs {
    /// Set data directory. Overrides config file value.
    #[arg(short, long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// Use in RAM mode for SQLite.
    #[arg(short, long)]
    pub sqlite_in_ram: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ImageProcessModeArgs {
    /// Input image file.
    #[arg(long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(long, value_name = "TYPE")]
    pub input_file_type: InputFileType,

    /// Output jpeg image file. Will be overwritten if exists.
    #[arg(long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum InputFileType {
    JpegImage,
}

impl InputFileType {
    pub fn to_cmd_arg_value(&self) -> String {
        self.to_possible_value()
            .expect("Input file type variant hidden by mistake")
            .get_name()
            .to_string()
    }
}
