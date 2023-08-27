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
    /// Run test, benchmark or bot mode
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
    #[command(flatten)]
    pub server: ServerConfig,

    /// Directory for random man images
    #[arg(long, value_name = "DIR")]
    pub images_man: Option<PathBuf>,

    /// Directory for random woman images
    #[arg(long, value_name = "DIR")]
    pub images_woman: Option<PathBuf>,

    // Boolean flags

    /// Do not remove created database files
    #[arg(long)]
    pub no_clean: bool,

    /// Do not start new server instances
    #[arg(long)]
    pub no_servers: bool,

    /// First error quits
    #[arg(long)]
    pub early_quit: bool,

    #[command(subcommand)]
    pub mode: TestModeSubMode,
}

impl TestMode {
    pub fn bots(&self) -> u32 {
        match &self.mode {
            TestModeSubMode::Bot(c) => c.bots,
            TestModeSubMode::Benchmark(c) => c.bots,
            _ => 1,
        }
    }

    pub fn tasks(&self) -> u32 {
        match &self.mode {
            TestModeSubMode::Benchmark(c) => c.tasks,
            _ => 1,
        }
    }

    pub fn save_state(&self) -> bool {
        match &self.mode {
            TestModeSubMode::Bot(c) => c.save_state,
            TestModeSubMode::Benchmark(c) => c.save_state,
            _ => false,
        }
    }

    pub fn no_sleep(&self) -> bool {
        match &self.mode {
            TestModeSubMode::Bot(c) => c.no_sleep,
            TestModeSubMode::Benchmark(c) => !c.sleep,
            _ => false,
        }
    }

    pub fn qa_mode(&self) -> Option<&QaTestConfig> {
        match &self.mode {
            TestModeSubMode::Qa(c) => Some(c),
            _ => None,
        }
    }

    pub fn bot_mode(&self) -> Option<&BotConfig> {
        match &self.mode {
            TestModeSubMode::Bot(c) => Some(c),
            _ => None,
        }
    }

    pub fn selected_benchmark(&self) -> Option<&SelectedBenchmark> {
        match &self.mode {
            TestModeSubMode::Benchmark(c) => Some(&c.benchmark),
            _ => None,
        }
    }

    /// Test name which does not have whitespace
    pub fn test_name(&self) -> String {
        match &self.mode {
            TestModeSubMode::Bot(_) => format!("bot"),
            TestModeSubMode::Qa(_) => format!("qa"),
            TestModeSubMode::Benchmark(c) =>
                format!("benchmark_{:?}", c.benchmark),
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub enum TestModeSubMode {
    /// Run benchmark
    Benchmark(BenchmarkConfig),
    /// Run QA test suite
    Qa(QaTestConfig),
    /// Run bot mode
    Bot(BotConfig),
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

#[derive(Args, Debug, Clone)]
pub struct BenchmarkConfig {
    /// Bot count per task
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub bots: u32,

    /// Task count
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub tasks: u32,

    /// Enable bot sleep time
    #[arg(long)]
    pub sleep: bool,

    /// Select benchmark
    #[arg(long, default_value = "get-profile", value_name = "NAME", value_enum)]
    pub benchmark: SelectedBenchmark,

    /// Save and load state
    #[arg(long)]
    pub save_state: bool,
}

#[derive(Args, Debug, Clone)]
pub struct QaTestConfig;

#[derive(Args, Debug, Clone)]
pub struct BotConfig {
    /// Bot count per task
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub bots: u32,

    /// Make bots to make requests constantly
    #[arg(long)]
    pub no_sleep: bool,

    /// Save and load state
    #[arg(long)]
    pub save_state: bool,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum SelectedBenchmark {
    GetProfile,
    GetProfileFromDatabase,
    GetProfileList,
    PostProfile,
    PostProfileToDatabase,
}

impl SelectedBenchmark {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
