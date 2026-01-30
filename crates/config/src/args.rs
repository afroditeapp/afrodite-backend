//! Config given as command line arguments

use std::{fmt, num::NonZeroU8, path::PathBuf};

use clap::{Args, Parser, ValueEnum};
use error_stack::ResultExt;
use manager_config::args::ManagerApiClientMode;
use reqwest::Url;
use simple_backend_config::args::ServerMode;
use simple_backend_utils::{
    ContextExt, dir::abs_path_for_directory_or_file_which_might_not_exists,
};

use crate::{bot_config_file::BotConfigFile, file::ConfigFileError};

#[derive(Args, Debug, Clone)]
pub struct ArgsConfig {
    #[command(subcommand)]
    pub mode: AppMode,
}

#[derive(Parser, Debug, Clone)]
pub enum AppMode {
    /// Server mode
    Server(ServerMode),
    /// Run remote bot mode
    RemoteBot(RemoteBotMode),
    /// Run test, benchmark or bot mode
    Test(TestMode),
    /// Process received image - internal use only
    ImageProcess,
    /// Print API documentation JSON to stdout
    OpenApi,
    /// Manager mode
    Manager(ManagerMode),
    /// Manager API client mode
    ManagerApi(ManagerApiClientMode),
    /// Config related commands
    Config {
        #[command(subcommand)]
        mode: ConfigMode,
    },
    /// Server data related commands
    Data(DataMode),
    /// Print build info and quit
    BuildInfo,
}

#[derive(Args, Debug, Clone)]
pub struct ManagerMode {
    /// Path to manager config file. Default file is created if it
    /// doesn't exist. Working directory changes to where the file
    /// is located to make file paths in the config file relative
    /// from the config file.
    #[arg(long, value_name = "FILE")]
    pub manager_config: PathBuf,
}

#[derive(Parser, Debug, Clone)]
pub struct PublicApiUrl {
    /// Base URL for API
    #[arg(long, default_value = "http://127.0.0.1:3001", value_name = "URL")]
    pub api_url: Url,
}

impl PublicApiUrl {
    pub fn new(url: Url) -> Self {
        Self {
            api_url: url.clone(),
        }
    }

    #[allow(clippy::result_unit_err)]
    pub fn change_ports(mut self, port: Option<u16>) -> Result<Self, ()> {
        if let Some(port) = port {
            self.api_url.set_port(Some(port))?;
        }
        Ok(self)
    }
}

#[derive(Args, Debug, Clone)]
pub struct RemoteBotMode {
    /// Working directory changes where the bot config is located, so
    /// file paths are relative from config file's directory.
    #[arg(long, value_name = "FILE")]
    pub bot_config: PathBuf,
}

impl RemoteBotMode {
    pub fn to_test_mode(&self) -> error_stack::Result<TestMode, ConfigFileError> {
        let bot_config_path_abs =
            abs_path_for_directory_or_file_which_might_not_exists(&self.bot_config)
                .change_context(ConfigFileError::LoadConfig)?;
        let config = BotConfigFile::load(&bot_config_path_abs, false)?;
        let Some(remote_bot_mode_config) = config.remote_bot_mode else {
            return Err(ConfigFileError::InvalidConfig.report())
                .attach_printable("Remote bot mode config not found");
        };

        Ok(TestMode {
            server: ServerConfig::default(),
            api_urls: PublicApiUrl::new(remote_bot_mode_config.api_url),
            bot_config: Some(bot_config_path_abs),
            data_dir: None,
            no_clean: false,
            no_servers: true,
            sqlite_in_ram: false,
            no_tmp_dir: false,
            mode: TestModeSubMode::Bot(BotModeConfig { save_state: false }),
        })
    }
}

#[derive(Args, Debug, Clone)]
pub struct TestMode {
    #[command(flatten)]
    pub server: ServerConfig,

    #[command(flatten)]
    pub api_urls: PublicApiUrl,

    /// Working directory changes where the bot config is located, so
    /// file paths are relative from config file's directory.
    #[arg(long, value_name = "FILE")]
    pub bot_config: Option<PathBuf>,

    /// Directory for test mode files
    #[arg(long, default_value = "test_data", value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    // Boolean flags
    /// Do not remove server instance files
    #[arg(long)]
    pub no_clean: bool,

    /// Do not start new server instances
    #[arg(long)]
    pub no_servers: bool,

    /// Do not use system temporary directory. Server instance files are
    /// written to data dir when this is set.
    #[arg(long)]
    pub no_tmp_dir: bool,

    /// Start servers using in RAM mode for SQLite
    #[arg(short, long)]
    pub sqlite_in_ram: bool,

    #[command(subcommand)]
    pub mode: TestModeSubMode,
}

impl TestMode {
    pub fn tasks(&self) -> u32 {
        match &self.mode {
            TestModeSubMode::Bot(_) => {
                // Bot count is now determined from the API, not from config.
                // This method is deprecated for bot mode. Use the bot accounts
                // retrieved from the get_bots API instead.
                1
            }
            TestModeSubMode::Benchmark(c) => match c.benchmark {
                SelectedBenchmark::GetProfileList => c.tasks + 1,
                _ => c.tasks,
            },
            TestModeSubMode::Qa(_) => panic!("QA test runner does not call this method"),
        }
    }

    pub fn save_state(&self) -> bool {
        match &self.mode {
            TestModeSubMode::Bot(c) => c.save_state,
            _ => false,
        }
    }

    pub fn qa_mode(&self) -> Option<&QaTestConfig> {
        match &self.mode {
            TestModeSubMode::Qa(c) => Some(c),
            _ => None,
        }
    }

    pub fn bot_mode(&self) -> Option<&BotModeConfig> {
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

    pub fn overridden_index_cell_size(&self) -> Option<NonZeroU8> {
        match &self.mode {
            TestModeSubMode::Benchmark(c) => c.index_cell_square_km,
            _ => None,
        }
    }

    /// Test name which does not have whitespace
    pub fn test_name(&self) -> String {
        match &self.mode {
            TestModeSubMode::Bot(_) => "bot".to_string(),
            TestModeSubMode::Qa(_) => "qa".to_string(),
            TestModeSubMode::Benchmark(c) => format!("benchmark_{:?}", c.benchmark),
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
    Bot(BotModeConfig),
}

#[derive(Parser, Debug, Clone, Default)]
pub struct ServerConfig {
    /// Enable debug logging for server instances
    #[arg(long)]
    pub log_debug: bool,
}

#[derive(Args, Debug, Clone)]
pub struct BenchmarkConfig {
    /// Task count
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub tasks: u32,

    /// Select benchmark
    #[arg(long, default_value = "get-profile", value_name = "NAME", value_enum)]
    pub benchmark: SelectedBenchmark,

    /// Override index cell size value
    #[arg(long)]
    pub index_cell_square_km: Option<NonZeroU8>,
}

#[derive(Args, Debug, Clone)]
pub struct QaTestConfig {
    /// Try to continue from test which name contains this text
    #[arg(long)]
    pub continue_from: Option<String>,

    /// Task count. Default value is logical CPU count.
    #[arg(short, long, value_name = "COUNT")]
    pub tasks: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct BotModeConfig {
    /// Save and load state
    #[arg(long)]
    pub save_state: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum SelectedBenchmark {
    GetProfile,
    GetProfileFromDatabase,
    /// Tasks:
    ///  - Moderate images and add one profile to index
    ///  - Location index reader bots (tasks flag)
    GetProfileList,
    PostProfile,
    PostProfileToDatabase,
}

impl fmt::Display for SelectedBenchmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Args, Debug, Clone)]
pub struct DataMode {
    /// Data directory
    #[arg(long, default_value = "data", value_name = "DIR")]
    pub data_dir: PathBuf,

    /// Config directory
    #[arg(long, default_value = "config", value_name = "DIR")]
    pub config_dir: PathBuf,

    #[command(subcommand)]
    pub mode: DataModeSubMode,
}

#[derive(Parser, Debug, Clone)]
pub enum DataModeSubMode {
    /// View data from database
    View {
        #[command(subcommand)]
        mode: DataViewSubMode,
    },
    /// Load data to database
    Load {
        #[command(subcommand)]
        mode: DataLoadSubMode,
    },
}

#[derive(Parser, Debug, Clone)]
pub enum DataLoadSubMode {
    /// Load bot config from file
    BotConfig {
        /// Path to bot config file
        file: PathBuf,
    },
    /// Load image processing config from file
    ImageProcessingConfig {
        /// Path to image processing config file
        file: PathBuf,
    },
}

#[derive(Parser, Debug, Clone)]
pub enum DataViewSubMode {
    /// View bot config
    BotConfig,
    /// View image processing config
    ImageProcessingConfig,
}

#[derive(Parser, Debug, Clone)]
pub enum ConfigMode {
    /// Check config
    Check {
        #[command(subcommand)]
        mode: ConfigCheckMode,
    },
    /// View config
    View {
        #[command(subcommand)]
        mode: ConfigViewMode,
    },
}

#[derive(Parser, Debug, Clone)]
pub enum ConfigCheckMode {
    /// Check server config
    Server {
        /// Server config dir
        dir: PathBuf,
    },
    /// Check manager config
    Manager {
        /// Path to manager config file
        file: PathBuf,
    },
    /// Check bot config
    Bot {
        /// Path to bot config file
        file: PathBuf,
    },
}

#[derive(Parser, Debug, Clone)]
pub enum ConfigViewMode {
    /// View server config
    Server {
        /// Server config dir
        dir: PathBuf,
    },
    /// View manager config
    Manager {
        /// Path to manager config file
        file: PathBuf,
    },
    /// View bot config
    Bot {
        /// Path to bot config file
        file: PathBuf,
    },
    /// View available profile index sizes using
    /// dimensions from config file
    IndexInfo {
        /// Server config dir
        dir: PathBuf,
    },
}
