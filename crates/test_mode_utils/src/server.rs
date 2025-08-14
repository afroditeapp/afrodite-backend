use std::{
    env, num::NonZeroU8, os::unix::process::CommandExt, path::PathBuf, process::Stdio, sync::Arc,
};

use config::{
    Config,
    args::{SelectedBenchmark, TestMode},
    file::{
        ApiConfig, AutomaticProfileSearchConfig, CONFIG_FILE_NAME, Components, ConfigFile,
        ConfigFileConfig, EmailAddress, ExternalServices, GrantAdminAccessConfig, LocationConfig,
    },
};
use nix::{sys::signal::Signal, unistd::Pid};
use reqwest::Url;
use server_data::index::info::LocationIndexInfoCreator;
use simple_backend_config::file::{
    DataConfig, GeneralConfig, IpInfoConfig, SimpleBackendConfigFile, SocketConfig,
    VideoCallingConfig,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead},
    process::Child,
    sync::Mutex,
    task::JoinHandle,
};
use tracing::{info, warn};

use crate::dir::DataDirUtils;

pub const TEST_ADMIN_ACCESS_EMAIL: &str = "admin@example.com";

pub const SERVER_INSTANCE_DIR_START: &str = "server_instance_";

pub const DEFAULT_LOCATION_CONFIG: LocationConfig = LocationConfig {
    latitude_top_left: 70.1,
    longitude_top_left: 19.5,
    latitude_bottom_right: 59.8,
    longitude_bottom_right: 31.58,
    index_cell_square_km: NonZeroU8::MAX,
};

pub const DEFAULT_LOCATION_CONFIG_BENCHMARK: LocationConfig = LocationConfig {
    index_cell_square_km: match NonZeroU8::new(1) {
        Some(value) => value,
        None => panic!(),
    },
    ..DEFAULT_LOCATION_CONFIG
};

#[derive(Debug, Clone, Default)]
pub struct AdditionalSettings {
    /// Store logs in RAM instead of using standard output or error.
    pub log_to_memory: bool,
    pub account_server_api_port: Option<u16>,
}

pub struct ServerManager {
    servers: Vec<ServerInstance>,
    config: Arc<TestMode>,
}

impl ServerManager {
    pub async fn new(
        all_config: &Config,
        config: Arc<TestMode>,
        settings: Option<AdditionalSettings>,
    ) -> Self {
        let settings = settings.unwrap_or_default();

        let dir = DataDirUtils::create_data_dir_if_needed(&config);

        let check_host = |url: &Url, name| {
            let host = url.host_str().unwrap();
            if !(host == "127.0.0.1" || host == "localhost") {
                panic!("{name} address was not 127.0.0.1. value: {host}");
            }
        };
        check_host(&config.api_urls.url_account, "account server");
        check_host(&config.api_urls.url_profile, "profile server");
        check_host(&config.api_urls.url_media, "media server");
        check_host(&config.api_urls.url_chat, "chat server");

        let bot_api_port = settings
            .account_server_api_port
            .unwrap_or(config.api_urls.url_account.port().unwrap());
        let account_config = new_config(&config, bot_api_port, Components::all_enabled(), None);
        let servers = vec![
            ServerInstance::new(
                dir.clone(),
                all_config,
                account_config,
                &config,
                settings.clone(),
            )
            .await,
        ];

        if config.server.microservice_profile
            || config.server.microservice_media
            || config.server.microservice_chat
        {
            warn!("Starting server in microservice mode is unsupported");
        }

        // TODO(microservice): Start microservice server instances

        Self { servers, config }
    }

    pub async fn close(self) {
        for s in self.servers {
            s.close_and_maybe_remove_data(!self.config.no_clean).await;
        }
    }

    pub async fn logs(&self) -> Vec<String> {
        let mut logs = Vec::new();
        for (i, s) in self.servers.iter().enumerate() {
            logs.push(format!("Server {i} logs:\n"));
            logs.extend(s.logs().await);
        }
        logs
    }

    pub async fn logs_string(&self) -> String {
        self.logs().await.join("\n")
    }
}

fn new_config(
    config: &TestMode,
    bot_api_port: u16,
    components: Components,
    external_services: Option<ExternalServices>,
) -> (ConfigFile, SimpleBackendConfigFile) {
    let config = ConfigFile {
        grant_admin_access: GrantAdminAccessConfig {
            debug_for_every_matching_new_account: false,
            debug_match_only_email_domain: false,
            email: EmailAddress(TEST_ADMIN_ACCESS_EMAIL.to_string()),
        }
        .into(),
        general: Default::default(),
        api: ApiConfig::default(),
        config_files: ConfigFileConfig::default(),
        automatic_profile_search: AutomaticProfileSearchConfig::default(),
        components: Some(components),
        location: if let Some(SelectedBenchmark::GetProfileList) = config.selected_benchmark() {
            let mut location = DEFAULT_LOCATION_CONFIG_BENCHMARK;
            if let Some(index_cell_size) = config.overridden_index_cell_size() {
                location.index_cell_square_km = index_cell_size;
            }
            info!(
                "{}",
                LocationIndexInfoCreator::new(location.clone())
                    .create_one(location.index_cell_square_km)
            );
            location
        } else {
            DEFAULT_LOCATION_CONFIG
        }
        .into(),
        external_services,
        demo_account: None,
        limits: None,
        profile_name_allowlist: None,
        remote_bot: None,
    };

    let simple_backend_config = SimpleBackendConfigFile {
        general: GeneralConfig {
            debug: Some(true),
            debug_face_detection_result: Some(true),
            log_timestamp: None,
        },
        data: DataConfig::default(),
        socket: SocketConfig {
            public_api: None,
            public_bot_api: None,
            local_bot_api_port: Some(bot_api_port),
            debug_local_bot_api_ip: None,
            // TODO(microservice): Configure internal API properly
            experimental_internal_api: None,
        },
        sign_in_with_apple: None,
        sign_in_with_google: None,
        manager: None,
        tls: None,
        lets_encrypt: None,
        tile_map: None,
        firebase_cloud_messaging: None,
        email_sending: None,
        scheduled_tasks: None,
        static_file_package_hosting: None,
        image_processing: None,
        ip_info: IpInfoConfig::default(),
        video_calling: VideoCallingConfig::default(),
    };

    (config, simple_backend_config)
}

pub struct ServerInstance {
    server: Child,
    dir: PathBuf,
    stdout_task: JoinHandle<()>,
    stderr_task: JoinHandle<()>,
    logs: Arc<Mutex<Vec<String>>>,
}

impl ServerInstance {
    pub async fn new(
        dir: PathBuf,
        all_config: &Config,
        (server_config, simple_backend_config): (ConfigFile, SimpleBackendConfigFile),
        args_config: &TestMode,
        settings: AdditionalSettings,
    ) -> Self {
        let id = simple_backend_utils::UuidBase64Url::new_random_id();
        let dir = dir.join(format!(
            "{}{}_{}",
            SERVER_INSTANCE_DIR_START,
            chrono::Utc::now(),
            id,
        ));
        std::fs::create_dir(&dir).unwrap();

        let config = toml::to_string_pretty(&server_config).unwrap();
        std::fs::write(dir.join(CONFIG_FILE_NAME), config).unwrap();

        let config = toml::to_string_pretty(&simple_backend_config).unwrap();
        std::fs::write(
            dir.join(simple_backend_config::file::CONFIG_FILE_NAME),
            config,
        )
        .unwrap();

        let start_cmd = env::args().next().unwrap();
        let start_cmd = std::fs::canonicalize(&start_cmd).unwrap();

        if !start_cmd.is_file() {
            panic!("First argument does not point to a file {:?}", &start_cmd);
        }

        let log_value = if args_config.server.log_debug {
            "debug"
        } else {
            "warn"
        };

        let mut command = std::process::Command::new(start_cmd);
        command
            .current_dir(&dir)
            .env("RUST_LOG", log_value)
            .process_group(0);

        if all_config.simple_backend().sqlite_in_ram() {
            command.arg("--sqlite-in-ram");
        }

        let mut tokio_command: tokio::process::Command = command.into();

        tokio_command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut server = tokio_command.kill_on_drop(true).spawn().unwrap();

        let logs = Arc::new(Mutex::new(Vec::new()));
        let stdout = server.stdout.take().expect("Stdout handle is missing");
        let stderr = server.stderr.take().expect("Stderr handle is missing");
        let (start_sender, start_receiver) = tokio::sync::oneshot::channel::<()>();

        fn create_read_lines_task(
            stream: impl AsyncRead + Unpin + Send + 'static,
            stream_name: &'static str,
            logs: Arc<Mutex<Vec<String>>>,
            log_to_memory: bool,
            start_sender: Option<tokio::sync::oneshot::Sender<()>>,
        ) -> JoinHandle<()> {
            tokio::spawn(async move {
                let mut start_sender = start_sender;
                let mut line_stream = tokio::io::BufReader::new(stream).lines();
                loop {
                    let (line, stream_ended) = match line_stream.next_line().await {
                        Ok(Some(line)) => (line, false),
                        Ok(None) => (format!("Server {stream_name} closed"), true),
                        Err(e) => (format!("Server {stream_name} error: {e:?}"), true),
                    };

                    if let Some(sender) = start_sender.take() {
                        if line.contains(simple_backend::SERVER_START_MESSAGE) {
                            sender.send(()).unwrap();
                        } else {
                            start_sender = Some(sender);
                        }
                    }

                    if log_to_memory {
                        logs.lock().await.push(line);
                    } else {
                        println!("{line}");
                    }

                    if stream_ended {
                        break;
                    }
                }
            })
        }

        let stdout_task = create_read_lines_task(
            stdout,
            "stdout",
            logs.clone(),
            settings.log_to_memory,
            Some(start_sender),
        );
        let stderr_task =
            create_read_lines_task(stderr, "stderr", logs.clone(), settings.log_to_memory, None);

        tokio::select! {
            _ = start_receiver => (),
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Server did not start in 5 seconds");
            }
        }

        Self {
            server,
            dir,
            stdout_task,
            stderr_task,
            logs,
        }
    }

    async fn close_and_maybe_remove_data(mut self, remove: bool) {
        let id = self.server.id().unwrap();
        nix::sys::signal::kill(Pid::from_raw(id.try_into().unwrap()), Signal::SIGINT).unwrap(); // CTRL-C
        self.server.wait().await.unwrap();

        if remove {
            let dir = self.dir.file_name().unwrap().to_string_lossy();
            if dir.starts_with(SERVER_INSTANCE_DIR_START) {
                std::fs::remove_dir_all(self.dir).unwrap();
            } else {
                panic!("Not database instance dir {dir}");
            }
        }

        self.stdout_task.await.unwrap();
        self.stderr_task.await.unwrap();
    }

    pub async fn logs(&self) -> Vec<String> {
        self.logs.lock().await.clone()
    }
}
