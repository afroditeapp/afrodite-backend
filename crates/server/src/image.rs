//! Image process

use std::{env, os::unix::process::CommandExt, process::Stdio, path::Path, str::from_utf8};

use config::{Config};
use error_stack::ResultExt;
use model::BotConfig;
use nix::{unistd::Pid, sys::signal::Signal};
use tokio::{process::Child, task::JoinHandle, io::{AsyncRead, AsyncBufReadExt}};
use utils::ContextExt;

use tracing::error;

use error_stack::{Result};


#[derive(thiserror::Error, Debug)]
pub enum ImageProcessError {
    #[error("Launch command creation failed")]
    LaunchCommand,
    #[error("Starting image process failed")]
    StartProcess,
    #[error("Closing image process failed")]
    Close,

    #[error("Image processing failed")]
    ImageProcessingFailure,
}

/// Start this binary again running in image processing mode.
pub struct ImageProcess;

impl ImageProcess {
    pub async fn start_image_process(input: &Path, output: &Path) -> Result<(), ImageProcessError> {
        let start_cmd = env::args()
            .next()
            .ok_or(ImageProcessError::LaunchCommand.report())?
            .to_string();

        let start_cmd = std::fs::canonicalize(&start_cmd)
            .change_context(ImageProcessError::LaunchCommand)?;

        if !start_cmd.is_file() {
            return Err(ImageProcessError::LaunchCommand)
                .attach_printable(format!("First argument does not point to a file {:?}", start_cmd));
        }

        let input = std::fs::canonicalize(&input)
            .change_context(ImageProcessError::LaunchCommand)?;
        let output = std::fs::canonicalize(&output)
            .change_context(ImageProcessError::LaunchCommand)?;


        let mut command = std::process::Command::new(start_cmd);
        command
            .arg("image-process")
            .arg("--input")
            .arg(input)
            .arg("--output")
            .arg(output)
            .process_group(0);

        let mut tokio_command: tokio::process::Command = command.into();
        let result = tokio_command
            .kill_on_drop(true)
            .output()
            .await
            .change_context(ImageProcessError::StartProcess)?;

        if result.status.success() {
            Ok(())
        } else {
            let mut report = ImageProcessError::ImageProcessingFailure.report();
            let stdout_str = match from_utf8(&result.stdout) {
                Ok(msg) => msg.trim(),
                Err(_) => "stdout contains invalid utf-8",
            };
            if !stdout_str.is_empty() {
                report = report.attach_printable(format!("stdout: {}", stdout_str));
            }
            let stderr_str = match from_utf8(&result.stderr) {
                Ok(msg) => msg.trim(),
                Err(_) => "stderr contains invalid utf-8",
            };
            if !stderr_str.is_empty() {
                report = report.attach_printable(format!("stderr: {}", stderr_str));
            }
            Err(report)
        }
    }
}
