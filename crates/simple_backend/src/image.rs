//! Image process

use std::{
    env, os::unix::process::CommandExt, path::Path, process::Stdio, sync::OnceLock, time::Duration,
};

use error_stack::{Result, ResultExt};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_image_process::{ImageProcessingCommand, ImageProcessingInfo, InputFileType};
use simple_backend_utils::ContextExt;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    sync::Mutex,
    task::JoinHandle,
};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum ImageProcessError {
    #[error("Launch command creation failed")]
    LaunchCommand,
    #[error("Starting image process failed")]
    StartProcess,
    #[error("Stdout handle is missing")]
    StdoutHandleMissing,
    #[error("Stdin handle is missing")]
    StdinHandleMissing,
    #[error("Stderr handle is missing")]
    StderrHandleMissing,
    #[error("Command writing failed")]
    WriteCommand,
    #[error("Info reading failed")]
    ReadInfo,
    #[error("Reading timeout")]
    ReadTimeout,

    #[error("Image processing command creation failed")]
    ImageProcessingCommandCreationFailed,
}

fn get_image_process() -> &'static Mutex<Option<ImageProcessHandle>> {
    static IMAGE_PROCESS: OnceLock<Mutex<Option<ImageProcessHandle>>> = OnceLock::new();
    IMAGE_PROCESS.get_or_init(|| Mutex::new(None))
}

pub struct ImageProcessHandle {
    stdin: ChildStdin,
    stdout: ChildStdout,
    child: Child,
    stderr_reader: JoinHandle<()>,
}

impl ImageProcessHandle {
    pub async fn start(config: &SimpleBackendConfig) -> Result<Self, ImageProcessError> {
        let start_cmd = env::args()
            .next()
            .ok_or(ImageProcessError::LaunchCommand.report())?
            .to_string();

        let start_cmd =
            std::fs::canonicalize(&start_cmd).change_context(ImageProcessError::LaunchCommand)?;

        if !start_cmd.is_file() {
            return Err(ImageProcessError::LaunchCommand).attach_printable(format!(
                "First argument does not point to a file {start_cmd:?}"
            ));
        }

        let mut command = std::process::Command::new(start_cmd);
        command
            .arg("image-process")
            .arg("--simple-backend-config")
            .arg(config.config_file_path())
            .process_group(0)
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut tokio_command: tokio::process::Command = command.into();
        let mut child = tokio_command
            .kill_on_drop(true)
            .spawn()
            .change_context(ImageProcessError::StartProcess)?;

        #[cfg(unix)]
        if let Some(nice_value) = config.image_processing().process_nice_value {
            if let Some(pid) = child.id() {
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
                            "Failed to set nice value for image process: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to execute renice for image process: {}", e);
                    }
                }
            } else {
                error!("Failed to set nice value for image process: no PID value");
            }
        }

        let Some(stderr) = child.stderr.take() else {
            return Err(ImageProcessError::StderrHandleMissing.into());
        };

        let stderr_reader = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => warn!("Image process error: {}", line),
                    Ok(None) => break,
                    Err(e) => error!("Image process stderr reading error: {}", e),
                }
            }
        });

        let Some(stdin) = child.stdin.take() else {
            return Err(ImageProcessError::StdinHandleMissing.into());
        };

        let Some(stdout) = child.stdout.take() else {
            return Err(ImageProcessError::StdoutHandleMissing.into());
        };

        Ok(ImageProcessHandle {
            stderr_reader,
            stdin,
            stdout,
            child,
        })
    }

    async fn write_command(
        write: &mut ChildStdin,
        command: ImageProcessingCommand,
    ) -> Result<(), ImageProcessError> {
        let string =
            serde_json::to_string(&command).change_context(ImageProcessError::WriteCommand)?;
        let len = TryInto::<u32>::try_into(string.len())
            .change_context(ImageProcessError::WriteCommand)?;
        write
            .write_all(&len.to_le_bytes())
            .await
            .change_context(ImageProcessError::WriteCommand)?;
        write
            .write_all(string.as_bytes())
            .await
            .change_context(ImageProcessError::WriteCommand)?;
        write
            .flush()
            .await
            .change_context(ImageProcessError::WriteCommand)?;
        Ok(())
    }

    async fn read_info(read: &mut ChildStdout) -> Result<ImageProcessingInfo, ImageProcessError> {
        let mut length = [0; 4];
        read.read_exact(&mut length)
            .await
            .change_context(ImageProcessError::ReadInfo)?;
        let length = u32::from_le_bytes(length);
        let mut bytes: Vec<u8> = vec![0; length as usize];
        read.read_exact(&mut bytes)
            .await
            .change_context(ImageProcessError::ReadInfo)?;
        serde_json::from_reader(bytes.as_slice()).change_context(ImageProcessError::ReadInfo)
    }

    async fn run_command(
        mut self,
        command: ImageProcessingCommand,
    ) -> Result<(Self, ImageProcessingInfo), ImageProcessError> {
        let r = Self::write_command(&mut self.stdin, command).await;
        if let Err(e) = r {
            self.close().await;
            return Err(e);
        }

        let info = tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                self.close().await;
                return Err(ImageProcessError::ReadTimeout.into());
            }
            r = Self::read_info(&mut self.stdout) => {
                match r {
                    Ok(info) => info,
                    Err(e) => {
                        self.close().await;
                        return Err(e);
                    }
                }
            }
        };

        Ok((self, info))
    }

    async fn close(mut self) {
        if let Err(e) = self.child.kill().await {
            error!("Closing image process failed: {e}");
        }
        if let Err(e) = self.stderr_reader.await {
            error!("Closing image process stderr reader failed: {e}");
        }
    }
}

/// Start this binary again running in image processing mode if not
/// already running and process an image.
///
/// The server binary must implement the `image-process` subcommand.
///
/// Outputs JPEG images only.
pub struct ImageProcess;

impl ImageProcess {
    pub async fn start_image_process(
        config: &SimpleBackendConfig,
        input: &Path,
        input_file_type: InputFileType,
        output: &Path,
    ) -> Result<ImageProcessingInfo, ImageProcessError> {
        let input = std::fs::canonicalize(input)
            .change_context(ImageProcessError::ImageProcessingCommandCreationFailed)?;
        let output = if output.exists() {
            std::fs::canonicalize(output)
                .change_context(ImageProcessError::ImageProcessingCommandCreationFailed)?
        } else {
            let output_file_name = output
                .file_name()
                .ok_or(ImageProcessError::ImageProcessingCommandCreationFailed.report())?;
            if let Some(parent) = output.parent() {
                let path = std::fs::canonicalize(parent)
                    .change_context(ImageProcessError::ImageProcessingCommandCreationFailed)?;
                path.join(output_file_name)
            } else {
                return Err(ImageProcessError::ImageProcessingCommandCreationFailed.report())
                    .attach_printable(format!("Output path {output:?} has no parent"));
            }
        };

        let command = ImageProcessingCommand {
            input,
            input_file_type,
            output,
        };

        let mut state = get_image_process().lock().await;

        let handle = match state.take() {
            Some(handle) => handle,
            None => ImageProcessHandle::start(config).await?,
        };

        let (handle, info) = handle.run_command(command).await?;
        *state = Some(handle);

        Ok(info)
    }

    /// Close current image process if it exists
    pub async fn close() {
        if let Some(handle) = get_image_process().lock().await.take() {
            handle.close().await;
        }
    }
}
