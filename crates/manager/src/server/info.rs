//! Get system info

use std::process::ExitStatus;

use error_stack::{Result, ResultExt};
use manager_model::{CommandOutput, SystemInfo, SystemInfoList};
use tokio::process::Command;

use super::client::ApiManager;
use crate::config::Config;

#[derive(thiserror::Error, Debug)]
pub enum SystemInfoError {
    #[error("Process start failed")]
    ProcessStartFailed,

    #[error("Process wait failed")]
    ProcessWaitFailed,

    #[error("Process stdin writing failed")]
    ProcessStdinFailed,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),

    #[error("Invalid output")]
    InvalidOutput,

    #[error("Invalid input")]
    InvalidInput,

    #[error("Invalid path")]
    InvalidPath,

    #[error("Api request failed")]
    ApiRequest,
}

pub struct SystemInfoGetter;

impl SystemInfoGetter {
    pub async fn system_info_all(
        config: &Config,
        api: &ApiManager<'_>,
    ) -> Result<SystemInfoList, SystemInfoError> {
        let system_info = Self::system_info(config).await?;
        let mut system_infos = vec![system_info];

        if let Some(info_config) = config.system_info() {
            for service in info_config.remote_managers.iter().flatten() {
                match api.system_info(&service.name).await {
                    Ok(info) => {
                        let info = SystemInfo {
                            name: format!(
                                "Remote manager {}, remote name: {}",
                                service.name, info.name
                            ),
                            info: info.info,
                        };
                        system_infos.push(info);
                    }
                    Err(e) => {
                        tracing::error!("Failed to get system info from {}: {:?}", service.name, e);
                        let _error = e.to_string();
                    }
                }
            }
        }

        Ok(SystemInfoList { info: system_infos })
    }

    pub async fn system_info(config: &Config) -> Result<SystemInfo, SystemInfoError> {
        let df = Self::run_df().await?;
        let df_inodes = Self::run_df_inodes().await?;
        let uptime = Self::run_uptime().await?;
        let free = Self::run_free().await?;
        let print_logs = Self::run_print_logs(config).await?;

        let whoami = Self::run_whoami().await?;
        let username = whoami.output.trim().to_string();
        let top = Self::run_top(&username).await?;

        let mut commands = vec![df, df_inodes, uptime, free, top, print_logs];

        if let Some(info_config) = config.system_info() {
            for service in info_config.log_services.iter() {
                let journalctl = Self::run_journalctl(service).await?;
                commands.push(journalctl);
            }
        }

        let hostname = Self::run_hostname().await?;
        Ok(SystemInfo {
            name: hostname.output.trim().to_string(),
            info: commands,
        })
    }

    async fn run_df() -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("df", &["-h"]).await
    }

    async fn run_df_inodes() -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("df", &["-hi"]).await
    }

    async fn run_uptime() -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("uptime", &[]).await
    }

    async fn run_hostname() -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("hostname", &[]).await
    }

    async fn run_whoami() -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("whoami", &[]).await
    }

    async fn run_top(username: &str) -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("top", &["-bn", "1", "-u", username]).await
    }

    async fn run_free() -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("free", &["-h"]).await
    }

    async fn run_journalctl(service: &str) -> Result<CommandOutput, SystemInfoError> {
        Self::run_cmd_with_args("journalctl", &["--no-pager", "-n", "20", "-u", service]).await
    }

    /// Run print-logs.sh script which prints some logs requiring sudo.
    async fn run_print_logs(config: &Config) -> Result<CommandOutput, SystemInfoError> {
        let script = config.script_locations().print_logs();
        let script_str = script.to_str().ok_or(SystemInfoError::InvalidInput)?;
        Self::run_cmd_with_args("sudo", &[script_str]).await
    }

    async fn run_cmd_with_args(cmd: &str, args: &[&str]) -> Result<CommandOutput, SystemInfoError> {
        let output = Command::new(cmd)
            .args(args)
            .output()
            .await
            .change_context(SystemInfoError::ProcessWaitFailed)?;

        if !output.status.success() {
            tracing::error!(
                "{} {} failed with status: {:?}",
                cmd,
                args.join(" "),
                output.status
            );
            return Err(SystemInfoError::CommandFailed(output.status).into());
        }

        let output =
            String::from_utf8(output.stdout).change_context(SystemInfoError::InvalidOutput)?;

        Ok(CommandOutput {
            name: format!("{} {}", cmd, args.join(" ")),
            output,
        })
    }
}
