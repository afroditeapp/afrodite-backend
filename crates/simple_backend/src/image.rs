//! Image process

use std::{env, os::unix::process::CommandExt, path::Path, str::from_utf8};

use error_stack::{Result, ResultExt};
use simple_backend_config::args::InputFileType;
use simple_backend_utils::ContextExt;
use tracing::error;

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
///
/// The server binary must implement the `image-process` subcommand.
/// Argument struct can be found from
/// `simple_backend_config::args::ImageProcessModeArgs`.
///
/// Outputs JPEG images only.
pub struct ImageProcess;

impl ImageProcess {
    pub async fn start_image_process(input: &Path, input_file_type: InputFileType, output: &Path) -> Result<(), ImageProcessError> {
        let start_cmd = env::args()
            .next()
            .ok_or(ImageProcessError::LaunchCommand.report())?
            .to_string();

        let start_cmd =
            std::fs::canonicalize(&start_cmd).change_context(ImageProcessError::LaunchCommand)?;

        if !start_cmd.is_file() {
            return Err(ImageProcessError::LaunchCommand).attach_printable(format!(
                "First argument does not point to a file {:?}",
                start_cmd
            ));
        }

        let input =
            std::fs::canonicalize(&input).change_context(ImageProcessError::LaunchCommand)?;
        let output = if output.exists() {
            std::fs::canonicalize(&output).change_context(ImageProcessError::LaunchCommand)?
        } else {
            let output_file_name = output
                .file_name()
                .ok_or(ImageProcessError::LaunchCommand.report())?;
            if let Some(parent) = output.parent() {
                let path = std::fs::canonicalize(&parent)
                    .change_context(ImageProcessError::LaunchCommand)?;
                path.join(output_file_name)
            } else {
                return Err(ImageProcessError::LaunchCommand.report())
                    .attach_printable(format!("Output path {:?} has no parent", output));
            }
        };

        let mut command = std::process::Command::new(start_cmd);
        command
            .arg("image-process")
            .arg("--input")
            .arg(input)
            .arg("--input-file-type")
            .arg(input_file_type.to_cmd_arg_value())
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
