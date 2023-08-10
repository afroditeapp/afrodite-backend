use std::{path::Path, process::Stdio, sync::Arc};

use app_manager::utils::IntoReportExt;
use nix::{sys::signal::Signal, unistd::Pid};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead},
    process::{Child, Command},
    task::JoinHandle,
};
use tracing::log::{error, info};

use crate::data::DatabaseRoot;
use config::{file::LitestreamConfig, Config};

use error_stack::{Result, ResultExt};

#[derive(thiserror::Error, Debug)]
pub enum LitestreamError {
    #[error("Process start failed")]
    ProcessStart,
    #[error("Process wait failed")]
    ProcessWait,
    #[error("Process stdout missing")]
    ProcessStdoutMissing,
    #[error("Process stderr missing")]
    ProcessStderrMissing,

    #[error("Invalid process ID")]
    InvalidPid,
    #[error("Sending signal failed")]
    SendSignal,
    #[error("Invalid output")]
    InvalidOutput,
    #[error("Database directory related error")]
    DatabaseDirError,
    #[error("Close stdout failed")]
    CloseStdoutFailed,
    #[error("Close stderr failed")]
    CloseStderrFailed,
}

struct LitestreamProcess {
    process: Child,
    stdout_task: JoinHandle<()>,
    stderr_task: JoinHandle<()>,
}

/// Start and stop the Litestream process.
pub struct LitestreamManager {
    config: Arc<Config>,
    litestream_config: LitestreamConfig,
    process: Option<LitestreamProcess>,
}

impl LitestreamManager {
    pub fn new(config: Arc<Config>, litestream_config: LitestreamConfig) -> Self {
        Self {
            config,
            litestream_config,
            process: None,
        }
    }

    pub async fn start_litestream(&mut self) -> Result<(), LitestreamError> {
        let mut process = Command::new(&self.litestream_config.binary)
            .arg("replicate")
            .arg("--config")
            .arg(&self.litestream_config.config_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .into_error(LitestreamError::ProcessStart)?;

        let stdout = process
            .stdout
            .take()
            .ok_or(LitestreamError::ProcessStdoutMissing)?;
        let stderr = process
            .stderr
            .take()
            .ok_or(LitestreamError::ProcessStderrMissing)?;

        fn create_read_lines_task(
            stream: impl AsyncRead + Unpin + Send + 'static,
            stream_name: &'static str,
        ) -> JoinHandle<()> {
            tokio::spawn(async move {
                let mut line_stream = tokio::io::BufReader::new(stream).lines();
                loop {
                    match line_stream.next_line().await {
                        Ok(Some(line)) => {
                            info!("Litestream: {}", line);
                        }
                        Ok(None) => {
                            info!("Litestream {stream_name} closed");
                            break;
                        }
                        Err(e) => {
                            error!("Litestream {stream_name} error: {e:?}");
                            break;
                        }
                    }
                }
            })
        }

        let stdout_task = create_read_lines_task(stdout, "stdout");
        let stderr_task = create_read_lines_task(stderr, "stdout");
        self.process = Some(LitestreamProcess {
            process,
            stdout_task,
            stderr_task,
        });

        self.wait_litestream().await?;

        Ok(())
    }

    pub async fn stop_litestream(mut self) -> Result<(), LitestreamError> {
        if let Some(mut process) = self.process.take() {
            if let Some(pid) = process.process.id() {
                let pid = Pid::from_raw(pid.try_into().into_error(LitestreamError::InvalidPid)?);
                // Send CTRL-C
                nix::sys::signal::kill(pid, Signal::SIGINT)
                    .into_error(LitestreamError::SendSignal)?;

                // No timeout because server is already mostly closed and
                // systemd has timeout for closing the server.
                let status = process
                    .process
                    .wait()
                    .await
                    .into_error(LitestreamError::ProcessWait)?;
                if !status.success() {
                    error!("Litestream process exited with error, status: {:?}", status);
                }
            } else {
                error!("Litestream closed too early");
                let status = process
                    .process
                    .wait()
                    .await
                    .into_error(LitestreamError::ProcessWait)?;
                if !status.success() {
                    error!("Litestream process exited with error, status: {:?}", status);
                }
            }
            process
                .stderr_task
                .await
                .into_error(LitestreamError::CloseStderrFailed)?;
            process
                .stdout_task
                .await
                .into_error(LitestreamError::CloseStdoutFailed)?;
        }

        Ok(())
    }

    pub async fn wait_litestream(&self) -> Result<(), LitestreamError> {
        let root = DatabaseRoot::new(self.config.database_dir())
            .change_context(LitestreamError::DatabaseDirError)?;
        let current_db_path = root.current_db_file();
        let history_db_path = root.history_db_file();

        if current_db_path.exists() {
            if !self
                .opened_by_litestream_with_retry(&current_db_path)
                .await?
            {
                error!("Litestream did not open current database");
                return Ok(());
            }
        }

        if history_db_path.exists() {
            if !self
                .opened_by_litestream_with_retry(&history_db_path)
                .await?
            {
                error!("Litestream did not open history database");
                return Ok(());
            }
        }

        Ok(())
    }

    pub async fn opened_by_litestream_with_retry(
        &self,
        file: &Path,
    ) -> Result<bool, LitestreamError> {
        for _ in 0..5 {
            if self.opened_by_litestream(&file).await? {
                return Ok(true);
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        Ok(false)
    }

    pub async fn opened_by_litestream(&self, file: &Path) -> Result<bool, LitestreamError> {
        // It seems that lsof displays only nine characters from command by default.
        Ok(self.lsof_output(file).await?.contains("litestrea"))
    }

    pub async fn lsof_output(&self, file: &Path) -> Result<String, LitestreamError> {
        let output = Command::new("lsof")
            .arg(file)
            .output()
            .await
            .into_error(LitestreamError::ProcessWait)?;

        let stdout_string =
            String::from_utf8(output.stdout).into_error(LitestreamError::InvalidOutput)?;

        Ok(stdout_string)
    }
}
