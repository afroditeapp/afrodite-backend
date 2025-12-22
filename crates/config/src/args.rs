//! Config given as command line arguments

use std::{fmt, num::NonZeroU8, path::PathBuf};

use clap::{Args, Parser, ValueEnum};
use error_stack::ResultExt;
use manager_config::args::ManagerApiClientMode;
use reqwest::Url;
use simple_backend_config::args::ServerMode;
use simple_backend_utils::ContextExt;

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
    /// Process received image
    ImageProcess(ImageProcessMode),
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
    /// Print build info and quit
    BuildInfo,
}

#[derive(Args, Debug, Clone)]
pub struct ImageProcessMode {
    #[arg(long, value_name = "FILE")]
    pub simple_backend_config: PathBuf,
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
    #[arg(long, value_name = "FILE")]
    pub bot_config: PathBuf,
}

impl RemoteBotMode {
    pub fn to_test_mode(&self) -> error_stack::Result<TestMode, ConfigFileError> {
        let config = BotConfigFile::load(self.bot_config.clone())?;
        let Some(server_url) = config.remote_bot_mode.map(|v| v.api_url) else {
            return Err(ConfigFileError::InvalidConfig.report())
                .attach_printable("Remote bot mode config not found");
        };

        for b in &config.bots {
            if b.account_id.is_none() {
                return Err(ConfigFileError::InvalidConfig.report())
                    .attach_printable(format!("Account ID is missing from bot {}", b.id));
            }

            if b.remote_bot_login_password.is_none() {
                return Err(ConfigFileError::InvalidConfig.report()).attach_printable(format!(
                    "Remote bot login password is missing from bot {}",
                    b.id
                ));
            }
        }

        let admin_config = config.admin_bot_config;
        if admin_config.account_id.is_some() || admin_config.remote_bot_login_password.is_some() {
            if admin_config.account_id.is_none() {
                return Err(ConfigFileError::InvalidConfig.report())
                    .attach_printable("Account ID is missing from admin bot");
            }

            if admin_config.remote_bot_login_password.is_none() {
                return Err(ConfigFileError::InvalidConfig.report())
                    .attach_printable("Remote bot login password is missing from admin bot");
            }
        }

        Ok(TestMode {
            server: ServerConfig::default(),
            api_urls: PublicApiUrl::new(server_url),
            bot_config: Some(self.bot_config.clone()),
            data_dir: None,
            no_clean: false,
            no_servers: true,
            early_quit: false,
            sqlite_in_ram: false,
            mode: TestModeSubMode::Bot(BotModeConfig {
                users: TryInto::<u32>::try_into(config.bots.len())
                    .change_context(ConfigFileError::InvalidConfig)?,
                admin: admin_config.account_id.is_some()
                    && admin_config.remote_bot_login_password.is_some(),
                no_sleep: false,
                save_state: false,
                task_per_bot: false,
            }),
        })
    }
}

#[derive(Args, Debug, Clone)]
pub struct TestMode {
    #[command(flatten)]
    pub server: ServerConfig,

    #[command(flatten)]
    pub api_urls: PublicApiUrl,

    #[arg(long, value_name = "FILE")]
    pub bot_config: Option<PathBuf>,

    /// Directory for test mode files
    #[arg(long, default_value = "tmp_data", value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    // Boolean flags
    /// Do not remove server instance database files
    #[arg(long)]
    pub no_clean: bool,

    /// Do not start new server instances
    #[arg(long)]
    pub no_servers: bool,

    /// First error quits
    #[arg(long)]
    pub early_quit: bool,

    /// Start servers using in RAM mode for SQLite
    #[arg(short, long)]
    pub sqlite_in_ram: bool,

    #[command(subcommand)]
    pub mode: TestModeSubMode,
}

impl TestMode {
    pub fn bots(&self, task_id: u32) -> u32 {
        match &self.mode {
            TestModeSubMode::Bot(c) if c.task_per_bot => 1,
            TestModeSubMode::Bot(c) if task_id == 0 => c.users,
            TestModeSubMode::Bot(_) if task_id == 1 => 1, // Admin bot
            TestModeSubMode::Benchmark(c) => c.bots,
            _ => 1,
        }
    }

    pub fn tasks(&self) -> u32 {
        match &self.mode {
            TestModeSubMode::Bot(c) if c.task_per_bot && c.admin => c.users + 1,
            TestModeSubMode::Bot(c) if c.task_per_bot => c.users,
            TestModeSubMode::Bot(c) if c.admin => 2,
            TestModeSubMode::Bot(_) => 1,
            TestModeSubMode::Benchmark(c) => match c.benchmark {
                SelectedBenchmark::GetProfileList => c.tasks + 2,
                _ => c.tasks,
            },
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
    /// User bot count
    #[arg(short, long, default_value = "1", value_name = "COUNT")]
    pub users: u32,

    /// Admin bot enabled
    #[arg(short, long)]
    pub admin: bool,

    /// Make bots to make requests constantly
    #[arg(long)]
    pub no_sleep: bool,

    /// Save and load state
    #[arg(long)]
    pub save_state: bool,

    /// Run each bot in a separate task
    #[arg(long)]
    pub task_per_bot: bool,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum SelectedBenchmark {
    GetProfile,
    GetProfileFromDatabase,
    /// This benchmark uses one extra task for filling the location index
    /// with profiles and another for admin bot.
    /// Bot count controls how many bots are created just
    /// for that.
    GetProfileList,
    PostProfile,
    PostProfileToDatabase,
}

impl fmt::Display for SelectedBenchmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Parser, Debug, Clone)]
pub enum ConfigMode {
    /// Check config. This loads the config like normally (includes
    /// directory creation for example) but does not save default config files.
    Check {
        /// Try to read config files from this directory. Use current directory
        /// if the argument does not exists.
        dir: Option<PathBuf>,
    },
    /// View config. This loads the config like normally (includes
    /// directory creation for example) but does not save default config files.
    View {
        /// Try to read config files from this directory. Use current directory
        /// if the argument does not exists.
        dir: Option<PathBuf>,
    },
    /// Print available profile index sizes using
    /// dimensions from config file and quit.
    IndexInfo {
        /// Try to read config file from this directory. Use current directory
        /// if the argument does not exists.
        dir: Option<PathBuf>,
    },
}
