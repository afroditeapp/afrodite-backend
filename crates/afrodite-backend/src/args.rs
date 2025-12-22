use clap::Parser;
use config::args::ArgsConfig;

// Define main CLI arguments struct here, so that
// correct version and other information from Cargo.toml
// is added to CLI.

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(flatten)]
    args: ArgsConfig,
}

pub fn get_config() -> ArgsConfig {
    Cli::parse().args
}
