

use app_manager::api::model::{SystemInfoList, CommandOutput, SystemInfo, SoftwareInfo, BuildInfo, SoftwareOptions};
use manager_api_client::{
    apis::{configuration::Configuration, manager_api::{post_request_build_software, post_request_software_update}}, manual_additions::get_latest_software_fixed,
};
use axum::{
    routing::{get, post},
    Router,
};

use error_stack::{Result, ResultExt};

use hyper::StatusCode;

use tracing::{error, info};

use crate::{
    api::{
        self,
        model::{
            Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
            ProfileInternal,
        }, GetConfig,
    },
    config::InternalApiUrls,
    utils::IntoReportExt,
};

use crate::{api::model::ApiKey, config::Config};

use super::{
    app::AppState,
    database::{
        commands::WriteCommandRunnerHandle,
        read::ReadCommands,
        utils::{AccountIdManager, ApiKeyManager},
    },
};

#[derive(thiserror::Error, Debug)]
pub enum ManagerClientError {
    #[error("API request failed")]
    ApiRequest,

    #[error("API URL not configured")]
    ApiUrlNotConfigured,

    #[error("Client build failed")]
    ClientBuildFailed,

    #[error("Missing value")]
    MissingValue,

    #[error("Invalid value")]
    InvalidValue,
}

pub struct ManagerApiClient {
    manager: Option<Configuration>,
}

impl ManagerApiClient {
    pub fn new(config: &Config) -> Result<Self, ManagerClientError> {
        let mut client = reqwest::ClientBuilder::new()
            .tls_built_in_root_certs(false);
        if let Some(cert) = config.root_certificate() {
            client = client.add_root_certificate(cert.clone());
        }
        let client = client.build().into_error(ManagerClientError::ClientBuildFailed)?;

        let manager = config.manager_config().map(|c| {
            let api_key = manager_api_client::apis::configuration::ApiKey {
                prefix: None,
                key: c.api_key.to_string(),
            };

            let url = c.address.as_str().trim_end_matches('/').to_string();

            info!("Manager API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                api_key: Some(api_key),
                ..Configuration::default()
            }
        });

        Ok(Self { manager })
    }

    pub fn manager(&self) -> Result<&Configuration, ManagerClientError> {
        self.manager
            .as_ref()
            .ok_or(ManagerClientError::ApiUrlNotConfigured.into())
    }
}

pub struct ManagerApiManager<'a> {
    config: &'a Config,
    api_client: &'a ManagerApiClient,
}

impl<'a> ManagerApiManager<'a> {
    pub fn new(
        config: &'a Config,
        api_client: &'a ManagerApiClient,
    ) -> Self {
        Self {
            config,
            api_client,
        }
    }

    pub async fn system_info(&self) -> Result<SystemInfoList, ManagerClientError> {
        let system_info = manager_api_client::apis::manager_api::get_system_info_all(
            self.api_client.manager()?,
        ).await.into_error(ManagerClientError::ApiRequest)?;

        let info_vec = system_info.info
            .into_iter()
            .map(|info| {
                let cmd_vec = info.info.into_iter().map(|info|
                    CommandOutput {
                        name: info.name,
                        output: info.output,
                    }
                ).collect::<Vec<CommandOutput>>();
                SystemInfo {
                    name: info.name,
                    info: cmd_vec,
                }
            })
            .collect::<Vec<SystemInfo>>();

        Ok(SystemInfoList { info: info_vec })
    }

    pub async fn software_info(&self) -> Result<SoftwareInfo, ManagerClientError> {
        let info = manager_api_client::apis::manager_api::get_software_info(
            self.api_client.manager()?,
        ).await.into_error(ManagerClientError::ApiRequest)?;

        let info_vec = info.current_software
            .into_iter()
            .map(|info| {
                BuildInfo {
                    commit_sha: info.commit_sha,
                    build_info: info.build_info,
                    name: info.name,
                    timestamp: info.timestamp,
                }
            })
            .collect::<Vec<BuildInfo>>();

        Ok(SoftwareInfo { current_software: info_vec })
    }

    pub async fn request_backend_update(&self) -> Result<SoftwareInfo, ManagerClientError> {
        let info = manager_api_client::apis::manager_api::get_software_info(
            self.api_client.manager()?,
        ).await.into_error(ManagerClientError::ApiRequest)?;

        let info_vec = info.current_software
            .into_iter()
            .map(|info| {
                BuildInfo {
                    commit_sha: info.commit_sha,
                    build_info: info.build_info,
                    name: info.name,
                    timestamp: info.timestamp,
                }
            })
            .collect::<Vec<BuildInfo>>();

        Ok(SoftwareInfo { current_software: info_vec })
    }

    async fn get_latest_build_info_raw(
        &self,
        options: SoftwareOptions,
    ) -> Result<Vec<u8>, ManagerClientError> {
        let converted_options = match options {
            SoftwareOptions::Manager => manager_api_client::models::SoftwareOptions::Manager,
            SoftwareOptions::Backend => manager_api_client::models::SoftwareOptions::Backend,
        };

        get_latest_software_fixed(
            self.api_client.manager()?,
            converted_options,
            manager_api_client::models::DownloadType::Info,
        ).await.into_error(ManagerClientError::ApiRequest)
    }

    pub async fn get_latest_build_info(
        &self,
        options: SoftwareOptions,
    ) -> Result<BuildInfo, ManagerClientError> {
        let info_json = self.get_latest_build_info_raw(options).await?;
        let info: BuildInfo = serde_json::from_slice(&info_json)
            .into_error(ManagerClientError::InvalidValue)?;
        Ok(info)
    }

    pub async fn request_build_software_from_build_server(
        &self,
        options: SoftwareOptions,
    ) -> Result<(), ManagerClientError> {
        let converted_options = match options {
            SoftwareOptions::Manager => manager_api_client::models::SoftwareOptions::Manager,
            SoftwareOptions::Backend => manager_api_client::models::SoftwareOptions::Backend,
        };

        post_request_build_software(
            self.api_client.manager()?,
            converted_options,
        ).await.into_error(ManagerClientError::ApiRequest)
    }

    pub async fn request_update_software(
        &self,
        options: SoftwareOptions,
        reboot: bool,
    ) -> Result<(), ManagerClientError> {
        let converted_options = match options {
            SoftwareOptions::Manager => manager_api_client::models::SoftwareOptions::Manager,
            SoftwareOptions::Backend => manager_api_client::models::SoftwareOptions::Backend,
        };

        post_request_software_update(
            self.api_client.manager()?,
            converted_options,
            reboot,
        ).await.into_error(ManagerClientError::ApiRequest)
    }
}
