//! Bot client

use std::{env, os::unix::process::CommandExt, process::Stdio};

use config::{file_dynamic::BotConfig, Config};
use error_stack::ResultExt;
use nix::{unistd::Pid, sys::signal::Signal};
use tokio::{process::Child, task::JoinHandle, io::{AsyncRead, AsyncBufReadExt}};
use utils::ContextExt;

use tracing::{info, error};

use error_stack::{Result};

const BOT_DATA_DIR_NAME: &str = "bots";

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
    pub async fn start_bots(config: &Config, bot_config: &BotConfig) -> Result<Self, BotClientError> {
        let start_cmd = env::args()
            .next()
            .ok_or(BotClientError::LaunchCommand.report())?
            .to_string();

        let start_cmd = std::fs::canonicalize(&start_cmd)
            .change_context(BotClientError::LaunchCommand)?;

        if !start_cmd.is_file() {
            return Err(BotClientError::LaunchCommand)
                .attach_printable(format!("First argument does not point to a file {:?}", start_cmd));
        }

        let bot_data_dir = config.database_dir().join(BOT_DATA_DIR_NAME);

        // TODO: Bot client HTTPS support

        let mut command = std::process::Command::new(start_cmd);
        command
            .arg("test")
            .arg("--test-database")
            .arg(bot_data_dir)
            .arg("--no-servers")
            // Urls
            .arg("--url-register")
            .arg(Self::internal_api_url(config))
            .arg("--url-account")
            .arg(Self::public_api_url(config))
            .arg("--url-profile")
            .arg(Self::public_api_url(config))
            .arg("--url-media")
            .arg(Self::public_api_url(config))
            .arg("--url-chat")
            .arg(Self::public_api_url(config))
            // Bot mode config
            .arg("bot")
            .arg("--save-state")
            .arg("--users")
            .arg(bot_config.users.to_string())
            .arg("--admins")
            .arg(bot_config.admins.to_string())
            .env("RUST_LOG", "info")
            .process_group(0);

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
                            println!("bot: {}", line);
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
            let pid = Pid::from_raw(TryInto::<i32>::try_into(pid).change_context(BotClientError::InvalidPid)?);
            // Send CTRL-C
            nix::sys::signal::kill(pid, Signal::SIGINT)
                .change_context(BotClientError::SendSignal)?;

            let status = self.bot_client
                .wait()
                .await
                .change_context(BotClientError::Close)?;
            if !status.success() {
                error!("Bot client process exited with error, status: {:?}", status);
            }
        } else {
            error!("Bot client closed too early");
            let status = self.bot_client
                .wait()
                .await
                .change_context(BotClientError::Close)?;
            if !status.success() {
                error!("Bot client process exited with error, status: {:?}", status);
            }
        }
        self
            .stderr_task
            .await
            .change_context(BotClientError::CloseStderrFailed)?;
        self
            .stdout_task
            .await
            .change_context(BotClientError::CloseStdoutFailed)?;
        Ok(())
    }

    fn internal_api_url(config: &Config) -> String {
        let port = config.socket().internal_api.port();
        format!("http://localhost:{}", port)
    }

    fn public_api_url(config: &Config) -> String {
        let port = config.socket().public_api.port();
        format!("http://localhost:{}", port)
    }
}
