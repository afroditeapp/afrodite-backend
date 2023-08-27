//! Config given as command line arguments

use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
    process::exit,
};

use clap::{arg, command, value_parser, Command, builder::PossibleValue, Parser, ValueEnum, Args};
use reqwest::Url;

#[derive(Args, Debug, Clone)]
pub struct ArgsConfig {
    /// Print build info and quit.
    #[arg(short, long)]
    pub build_info: bool,

    /// Set database directory. Overrides config file value.
    #[arg(short, long, value_name = "DIR")]
    pub database_dir: Option<PathBuf>,

    /// Use in RAM mode for SQLite.
    #[arg(short, long)]
    pub sqlite_in_ram: bool,

    #[command(subcommand)]
    pub test_mode: Option<AppMode>,
}

#[derive(Parser, Debug, Clone)]
pub enum AppMode {
    /// Run tests and benchmarks
    Test(TestMode),
}

#[derive(Parser, Debug, Clone)]
pub struct PublicApiUrls {
    /// Base URL for account API for register and login
    #[arg(long, default_value = "http://127.0.0.1:3001", value_name = "URL")]
    pub url_register: Url,

    /// Base URL for account API
    #[arg(long, default_value = "http://127.0.0.1:3000", value_name = "URL")]
    pub url_account: Url,

    /// Base URL for profile API
    #[arg(long, default_value = "http://127.0.0.1:3000", value_name = "URL")]
    pub url_profile: Url,

    /// Base URL for media API
    #[arg(long, default_value = "http://127.0.0.1:3000", value_name = "URL")]
    pub url_media: Url,

    /// Base URL for chat API
    #[arg(long, default_value = "http://127.0.0.1:3000", value_name = "URL")]
    pub url_chat: Url,
}

#[derive(Args, Debug, Clone)]
pub struct TestMode {
    /// Bot count per task
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub bots: u32,

    /// Task count
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub tasks: u32,

    #[command(flatten)]
    pub server: ServerConfig,

    /// Directory for random man images
    #[arg(long, value_name = "DIR")]
    pub images_man: Option<PathBuf>,

    /// Directory for random woman images
    #[arg(long, value_name = "DIR")]
    pub images_woman: Option<PathBuf>,

    // Boolean flags

    /// Make bots to make requests constantly
    #[arg(long)]
    pub no_sleep: bool,

    /// Do not remove created database files
    #[arg(long)]
    pub no_clean: bool,

    /// Do not start new server instances
    #[arg(long)]
    pub no_servers: bool,

    /// Save and load state
    #[arg(long)]
    pub save_state: bool,

    /// Update profile continuously
    /// TODO remove as there is also write benchmark?
    #[arg(long)]
    pub update_profile: bool,

    /// Print some speed information
    #[arg(long)]
    pub print_speed: bool,

    /// First error quits
    #[arg(long)]
    pub early_quit: bool,

    /// Run tests forever
    #[arg(long)]
    pub forever: bool,

    /// Select custom test
    #[arg(long, default_value = "qa", value_name = "NAME", value_enum)]
    pub test: Test,
}

#[derive(Parser, Debug, Clone)]
pub struct ServerConfig {
    #[command(flatten)]
    pub api_urls: PublicApiUrls,

    /// Directory for test database
    #[arg(long, default_value = "tmp_databases", value_name = "DIR")]
    pub test_database: PathBuf,

    /// Start media API as microservice
    #[arg(long)]
    pub microservice_media: bool,

    /// Start profile API as microservice
    #[arg(long)]
    pub microservice_profile: bool,

    /// Start chat API as microservice
    #[arg(long)]
    pub microservice_chat: bool,

    /// Enable debug logging for server instances
    #[arg(long)]
    pub log_debug: bool,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum Test {
    Qa,
    BenchmarkGetProfile,
    BenchmarkGetProfileFromDatabase,
    BenchmarkGetProfileList,
    BenchmarkPostProfile,
    BenchmarkPostProfileToDatabase,
    Bot,
}

impl Test {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
