//! Bot client

use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
    os::unix::process::CommandExt,
    process::Stdio,
};

use config::{Config, file::ConfigFile};
use error_stack::{Result, ResultExt};
use nix::{sys::signal::Signal, unistd::Pid};
use simple_backend_utils::ContextExt;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead},
    process::Child,
    task::JoinHandle,
};
use tracing::{error, info};

const BOT_DATA_DIR_NAME: &str = "bots";
const LOCALHOST_HOSTNAME: &str = "localhost";

#[derive(thiserror::Error, Debug)]
pub enum BotClientError {
    #[error("Launch command creation failed")]
    LaunchCommand,
    #[error("Starting bot client failed")]
    StartProcess,
    #[error("Closing bot client failed")]
    Close,

    #[error("Process stdout missing")]
    ProcessStdoutMissing,
    #[error("Process stderr missing")]
    ProcessStderrMissing,

    #[error("Invalid process ID")]
    InvalidPid,
    #[error("Sending signal failed")]
    SendSignal,
    #[error("Close stdout failed")]
    CloseStdoutFailed,
    #[error("Close stderr failed")]
    CloseStderrFailed,
}

/// Start this binary again running in bot mode.
pub struct BotClient {
    bot_client: Child,
    stdout_task: JoinHandle<()>,
    stderr_task: JoinHandle<()>,
}

impl BotClient {
    pub async fn start_bots(
        config: &Config,
        admin_bot: bool,
        user_bots: u32,
    ) -> Result<Self, BotClientError> {
        let start_cmd = env::args()
            .next()
            .ok_or(BotClientError::LaunchCommand.report())?
            .to_string();

        let start_cmd =
            std::fs::canonicalize(start_cmd).change_context(BotClientError::LaunchCommand)?;

        if !start_cmd.is_file() {
            return Err(BotClientError::LaunchCommand).attach_printable(format!(
                "First argument does not point to a file {start_cmd:?}"
            ));
        }

        let bot_api_socket = if let Some(port) = config.simple_backend().socket().local_bot_api_port
        {
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port)
        } else {
            return Err(BotClientError::LaunchCommand).attach_printable("Bot API must be enabled");
        };

        let bot_data_dir = config.simple_backend().data_dir().join(BOT_DATA_DIR_NAME);

        let mut command = std::process::Command::new(start_cmd);
        command
            .arg("test")
            .arg("--data-dir")
            .arg(bot_data_dir)
            .arg("--no-servers")
            // Urls
            .arg("--url-account")
            .arg(Self::bot_api_url(bot_api_socket))
            .arg("--url-profile")
            .arg(Self::bot_api_url(bot_api_socket))
            .arg("--url-media")
            .arg(Self::bot_api_url(bot_api_socket))
            .arg("--url-chat")
            .arg(Self::bot_api_url(bot_api_socket));

        if let Some(bot_config_file) = &config.bot_config_file() {
            let path = std::fs::canonicalize(bot_config_file)
                .change_context(BotClientError::LaunchCommand)?;
            command.arg("--bot-config-file").arg(path);
        }

        let server_config_path =
            ConfigFile::default_file_path().change_context(BotClientError::LaunchCommand)?;
        let path = std::fs::canonicalize(server_config_path)
            .change_context(BotClientError::LaunchCommand)?;
        command.arg("--server-config-file").arg(path);

        // Bot mode config
        command
            .arg("bot")
            .arg("--save-state")
            .arg("--users")
            .arg(user_bots.to_string());

        if admin_bot {
            command.arg("--admin");
        }

        // Setup logging and prevent signal propagation
        command.env("RUST_LOG", "info").process_group(0);

        let mut tokio_command: tokio::process::Command = command.into();
        let mut bot_client = tokio_command
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .change_context(BotClientError::StartProcess)?;

        let stdout = bot_client
            .stdout
            .take()
            .ok_or(BotClientError::ProcessStdoutMissing)?;
        let stderr = bot_client
            .stderr
            .take()
            .ok_or(BotClientError::ProcessStderrMissing)?;

        fn create_read_lines_task(
            stream: impl AsyncRead + Unpin + Send + 'static,
            stream_name: &'static str,
        ) -> JoinHandle<()> {
            tokio::spawn(async move {
                let mut line_stream = tokio::io::BufReader::new(stream).lines();
                loop {
                    match line_stream.next_line().await {
                        Ok(Some(line)) => {
                            println!("bot: {line}");
                        }
                        Ok(None) => {
                            info!("Bot client {stream_name} closed");
                            break;
                        }
                        Err(e) => {
                            error!("Bot client {stream_name} error: {e:?}");
                            break;
                        }
                    }
                }
            })
        }

        let stdout_task = create_read_lines_task(stdout, "stdout");
        let stderr_task = create_read_lines_task(stderr, "stderr");

        Ok(Self {
            bot_client,
            stdout_task,
            stderr_task,
        })
    }

    pub async fn stop_bots(mut self) -> Result<(), BotClientError> {
        if let Some(pid) = self.bot_client.id() {
            let pid = Pid::from_raw(
                TryInto::<i32>::try_into(pid).change_context(BotClientError::InvalidPid)?,
            );
            // Send CTRL-C
            nix::sys::signal::kill(pid, Signal::SIGINT)
                .change_context(BotClientError::SendSignal)?;

            let status = self
                .bot_client
                .wait()
                .await
                .change_context(BotClientError::Close)?;
            if !status.success() {
                error!("Bot client process exited with error, status: {:?}", status);
            }
        } else {
            error!("Bot client closed too early");
            let status = self
                .bot_client
                .wait()
                .await
                .change_context(BotClientError::Close)?;
            if !status.success() {
                error!("Bot client process exited with error, status: {:?}", status);
            }
        }
        self.stderr_task
            .await
            .change_context(BotClientError::CloseStderrFailed)?;
        self.stdout_task
            .await
            .change_context(BotClientError::CloseStdoutFailed)?;
        Ok(())
    }

    fn bot_api_url(api_socket: SocketAddr) -> String {
        format!("http://{}:{}", LOCALHOST_HOSTNAME, api_socket.port())
    }
}
