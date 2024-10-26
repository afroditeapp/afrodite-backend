use std::{
    env, net::SocketAddrV4, num::NonZeroU8, os::unix::process::CommandExt, path::PathBuf,
    process::Stdio, sync::Arc,
};

use config::{
    args::{SelectedBenchmark, TestMode},
    file::{
        Components, ConfigFile, EmailAddress, ExternalServices, GrantAdminAccessConfig,
        InternalApiConfig, LocationConfig, CONFIG_FILE_NAME,
    },
    Config,
};
use nix::{sys::signal::Signal, unistd::Pid};
use reqwest::Url;
use server_data::index::LocationIndexInfoCreator;
use simple_backend_config::file::{
    DataConfig, SimpleBackendConfigFile, SocketConfig, SqliteDatabase,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead},
    process::Child,
    sync::Mutex,
    task::JoinHandle,
};
use tracing::info;

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
    pub account_server_public_api_port: Option<u16>,
    pub account_server_internal_api_port: Option<u16>,
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

        let dir = config.server.test_database.clone();
        if !dir.exists() {
            std::fs::create_dir_all(&dir).unwrap();
        }

        let check_host = |url: &Url, name| {
            let host = url.host_str().unwrap();
            if !(host == "127.0.0.1" || host == "localhost") {
                panic!("{} address was not 127.0.0.1. value: {}", name, host);
            }
        };
        check_host(&config.server.api_urls.url_account, "account server");
        check_host(&config.server.api_urls.url_profile, "profile server");
        check_host(&config.server.api_urls.url_media, "media server");

        let account_port = config.server.api_urls.url_account.port().unwrap();
        let media_port = config.server.api_urls.url_media.port().unwrap();
        let profile_port = config.server.api_urls.url_profile.port().unwrap();

        let external_services = Some(ExternalServices {
            account_internal: format!("http://127.0.0.1:{}", account_port + 1)
                .parse::<Url>()
                .unwrap()
                .into(),
            media_internal: format!("http://127.0.0.1:{}", media_port + 1)
                .parse::<Url>()
                .unwrap()
                .into(),
        });

        let localhost_ip = "127.0.0.1".parse().unwrap();

        let account_public_api_port = settings
            .account_server_public_api_port
            .unwrap_or(account_port);
        let account_internal_api_port = settings
            .account_server_internal_api_port
            .unwrap_or(account_port + 1);
        let account_config = new_config(
            &config,
            SocketAddrV4::new(localhost_ip, account_public_api_port),
            SocketAddrV4::new(localhost_ip, account_internal_api_port),
            Components {
                account: true,
                profile: !config.server.microservice_profile,
                media: !config.server.microservice_media,
                chat: !config.server.microservice_chat,
            },
            external_services.clone(),
        );
        let mut servers = vec![
            ServerInstance::new(
                dir.clone(),
                all_config,
                account_config,
                &config,
                settings.clone(),
            )
            .await,
        ];

        if config.server.microservice_media {
            let server_config = new_config(
                &config,
                SocketAddrV4::new(localhost_ip, media_port),
                SocketAddrV4::new(localhost_ip, media_port + 1),
                Components {
                    media: true,
                    ..Components::default()
                },
                external_services.clone(),
            );
            servers.push(
                ServerInstance::new(
                    dir.clone(),
                    all_config,
                    server_config,
                    &config,
                    settings.clone(),
                )
                .await,
            );
        }

        if config.server.microservice_profile {
            let server_config = new_config(
                &config,
                SocketAddrV4::new(localhost_ip, profile_port),
                SocketAddrV4::new(localhost_ip, profile_port + 1),
                Components {
                    profile: true,
                    ..Components::default()
                },
                external_services,
            );
            servers.push(
                ServerInstance::new(
                    dir.clone(),
                    all_config,
                    server_config,
                    &config,
                    settings.clone(),
                )
                .await,
            );
        }

        // TODO: Chat microservice

        Self { servers, config }
    }

    pub async fn close(self) {
        for s in self.servers {
            s.close_and_maeby_remove_data(!self.config.no_clean).await;
        }
    }

    pub async fn logs(&self) -> Vec<String> {
        let mut logs = Vec::new();
        for (i, s) in self.servers.iter().enumerate() {
            logs.push(format!("Server {} logs:\n", i));
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
    public_api: SocketAddrV4,
    internal_api: SocketAddrV4,
    components: Components,
    external_services: Option<ExternalServices>,
) -> (ConfigFile, SimpleBackendConfigFile) {
    let config = ConfigFile {
        grant_admin_access: GrantAdminAccessConfig {
            for_every_matching_new_account: false,
            email: Some(EmailAddress(TEST_ADMIN_ACCESS_EMAIL.to_string())),
            google_account_id: None,
        }
        .into(),
        components,
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
        internal_api: InternalApiConfig {
            bot_login: true,
            // TODO(microservice): this should be enabled if microservice mode
            // is enabled
            microservice: false,
        }
        .into(),
        queue_limits: None,
        bot_config_file: None,
        profile_attributes_file: None,
        email_content_file: None,
        demo_mode: None,
        limits: None,
    };

    let simple_backend_config = SimpleBackendConfigFile {
        debug: Some(true),
        log_timestamp: None,
        data: DataConfig {
            dir: "database_dir".into(),
            sqlite: vec![
                SqliteDatabase {
                    name: "current".into(),
                },
                SqliteDatabase {
                    name: "history".into(),
                },
            ],
        },
        socket: SocketConfig {
            public_api: public_api.into(),
            internal_api: Some(internal_api.into()),
            internal_api_allow_non_localhost_ip: false,
        },
        sign_in_with_google: None,
        manager: None,
        tls: None,
        lets_encrypt: None,
        media_backup: None,
        litestream: None,
        tile_map: None,
        firebase_cloud_messaging: None,
        email_sending: None,
        scheduled_tasks: None,
        static_file_package_hosting: None,
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
        let id = uuid::Uuid::new_v4();
        let dir = dir.join(format!(
            "{}{}_{}",
            SERVER_INSTANCE_DIR_START,
            chrono::Utc::now(),
            id.hyphenated()
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

    async fn close_and_maeby_remove_data(mut self, remove: bool) {
        let id = self.server.id().unwrap();
        nix::sys::signal::kill(Pid::from_raw(id.try_into().unwrap()), Signal::SIGINT).unwrap(); // CTRL-C
        self.server.wait().await.unwrap();

        if remove {
            let dir = self.dir.file_name().unwrap().to_string_lossy();
            if dir.starts_with(SERVER_INSTANCE_DIR_START) {
                std::fs::remove_dir_all(self.dir).unwrap();
            } else {
                panic!("Not database instance dir {}", dir);
            }
        }

        self.stdout_task.await.unwrap();
        self.stderr_task.await.unwrap();
    }

    pub async fn logs(&self) -> Vec<String> {
        self.logs.lock().await.clone()
    }
}
