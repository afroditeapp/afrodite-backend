use std::process::ExitCode;

use clap::Parser;
use config::args::ArgsConfig;

use crate::build_info::build_info;

// Define main CLI arguments struct here, so that
// correct version and other information from Cargo.toml
// is added to CLI.

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(flatten)]
    pub args: ArgsConfig,
}

pub fn get_config() -> Result<ArgsConfig, ExitCode> {
    let matches = Cli::parse();

    if matches.args.build_info {
        println!("{}", build_info());
        Err(ExitCode::SUCCESS)
    } else {
        Ok(matches.args)
    }
}
