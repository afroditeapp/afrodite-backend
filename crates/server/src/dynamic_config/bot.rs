//! Bot client

use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
    os::unix::process::CommandExt,
    process::Stdio,
};

use config::Config;
use error_stack::{Result, ResultExt};
use nix::{sys::signal::Signal, unistd::Pid};
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
    pub async fn start_bots(config: &Config) -> Result<Self, BotClientError> {
        let current_exe = env::current_exe().change_context(BotClientError::LaunchCommand)?;

        let bot_api_socket = if let Some(port) = config.simple_backend().socket().local_bot_api_port
        {
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port)
        } else {
            return Err(BotClientError::LaunchCommand).attach_printable("Bot API must be enabled");
        };

        let bot_data_dir = config.simple_backend().data_dir().join(BOT_DATA_DIR_NAME);

        let mut command = std::process::Command::new(current_exe);
        command
            .arg("test")
            .arg("--data-dir")
            .arg(bot_data_dir)
            .arg("--no-servers")
            .arg("--api-url")
            .arg(Self::bot_api_url(bot_api_socket));

        command
            .arg("--bot-config")
            .arg(config.bot_config_abs_file_path());

        // Bot mode config
        command.arg("bot").arg("--save-state");

        // Setup logging and prevent signal propagation
        command.env("RUST_LOG", "info").process_group(0);

        let mut tokio_command: tokio::process::Command = command.into();
        let mut bot_client = tokio_command
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .change_context(BotClientError::StartProcess)?;

        #[cfg(unix)]
        if let Some(nice_value) = config.general().bot_process_nice_value {
            if let Some(pid) = bot_client.id() {
                let renice_result = tokio::process::Command::new("renice")
                    .arg("-n")
                    .arg(nice_value.to_string())
                    .arg("-p")
                    .arg(pid.to_string())
                    .output()
                    .await;

                match renice_result {
                    Ok(output) if !output.status.success() => {
                        error!(
                            "Failed to set nice value for bot process: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to execute renice for bot process: {}", e);
                    }
                }
            } else {
                error!("Failed to set nice value for bot process: no PID value");
            }
        }

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
