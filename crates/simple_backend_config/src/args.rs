//! Config given as command line arguments

use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
    process::exit,
};

use clap::{arg, builder::PossibleValue, command, value_parser, Args, Command, Parser, ValueEnum};
use reqwest::Url;

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
    #[arg(long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(long, value_name = "FILE")]
    pub output: PathBuf,

    /// Jpeg quality value. Value is clamped between 1-100.
    /// Mozjpeg library recommends 60-80 values
    #[arg(long, value_name = "NUMBER", default_value = "60")]
    pub quality: u8,
}
