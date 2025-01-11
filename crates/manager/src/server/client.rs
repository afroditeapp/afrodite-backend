use std::{collections::HashMap, time::Duration};

use error_stack::{Result, ResultExt};
use manager_api::{ApiKey, Configuration, ManagerApi};
use manager_model::{BuildInfo, DataEncryptionKey, SoftwareOptions, SystemInfo};
use tracing::info;

use crate::config::Config;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Client build failed")]
    ClientBuildFailed,

    #[error("API request failed")]
    ApiRequest,

    #[error("Database call failed")]
    DatabaseError,

    #[error("Manager API URL not configured for {0}")]
    ManagerApiUrlNotConfigured(&'static str),

    #[error("Missing value")]
    MissingValue,

    #[error("Invalid value")]
    InvalidValue,

    #[error("Missing configuration")]
    MissingConfiguration,
}

fn base_client_builder(config: &Config) -> reqwest::ClientBuilder {
    let client = reqwest::ClientBuilder::new().tls_built_in_root_certs(false);
    if let Some(cert) = config.root_certificate() {
        client.add_root_certificate(cert.clone())
    } else {
        client
    }
}

#[derive(Debug)]
pub struct ApiClient {
    encryption_key_provider: Option<Configuration>,
    software_update_provider: Option<Configuration>,
    system_info_remote_managers: HashMap<String, Configuration>,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self, ApiError> {
        let api_key = ApiKey {
            prefix: None,
            key: config.api_key().to_string(),
        };

        let client = base_client_builder(config)
            .build()
            .change_context(ApiError::ClientBuildFailed)?;

        let encryption_key_client = if let Some(timeout_seconds) = config
            .secure_storage_config()
            .and_then(|v| v.key_download_timeout_seconds)
        {
            base_client_builder(config)
                .timeout(Duration::from_secs(timeout_seconds.into()))
                .build()
                .change_context(ApiError::ClientBuildFailed)?
        } else {
            client.clone()
        };

        let encryption_key_provider = config.secure_storage_config().map(|url| {
            let url = url
                .manager_base_url
                .as_str()
                .trim_end_matches('/')
                .to_string();

            info!("encryption_key_provider API base url: {}", url);

            Configuration {
                base_path: url,
                client: encryption_key_client,
                api_key: Some(api_key.clone()),
                ..Configuration::default()
            }
        });

        let software_update_provider = config.software_update_provider().map(|url| {
            let url = url
                .manager_base_url
                .as_str()
                .trim_end_matches('/')
                .to_string();

            info!("software_update_provider API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                api_key: Some(api_key.clone()),
                ..Configuration::default()
            }
        });

        let mut system_info_remote_managers = HashMap::new();

        if let Some(info_config) = config.system_info() {
            for service in info_config.remote_managers.iter().flatten() {
                let url = service
                    .manager_base_url
                    .as_str()
                    .trim_end_matches('/')
                    .to_string();

                info!(
                    "system_info_remote_managers, name: {}, API base url: {}",
                    service.name, url
                );

                let configuration = Configuration {
                    base_path: url,
                    client: client.clone(),
                    api_key: Some(api_key.clone()),
                    ..Configuration::default()
                };

                system_info_remote_managers.insert(service.name.clone(), configuration);
            }
        }

        Ok(Self {
            encryption_key_provider,
            software_update_provider,
            system_info_remote_managers,
        })
    }

    pub fn encryption_key_provider_config(&self) -> Result<&Configuration, ApiError> {
        self.encryption_key_provider
            .as_ref()
            .ok_or(ApiError::ManagerApiUrlNotConfigured("encryption_key_provider_config").into())
    }

    pub fn software_update_provider_config(&self) -> Result<&Configuration, ApiError> {
        self.software_update_provider
            .as_ref()
            .ok_or(ApiError::ManagerApiUrlNotConfigured("software_update_provider_config").into())
    }

    pub fn system_info_remote_manager_config(
        &self,
        manager_name: &str,
    ) -> Result<&Configuration, ApiError> {
        self.system_info_remote_managers
            .get(manager_name)
            .ok_or(ApiError::ManagerApiUrlNotConfigured("system_info_remote_manager_config").into())
    }
}

pub struct ApiManager<'a> {
    config: &'a Config,
    api_client: &'a ApiClient,
}

impl<'a> ApiManager<'a> {
    pub fn new(config: &'a Config, api_client: &'a ApiClient) -> Self {
        Self { config, api_client }
    }

    pub async fn get_encryption_key(&self) -> Result<DataEncryptionKey, ApiError> {
        let provider = self
            .config
            .secure_storage_config()
            .ok_or(ApiError::MissingConfiguration)?;

        ManagerApi::get_encryption_key(
            self.api_client.encryption_key_provider_config()?,
            &provider.encryption_key_name,
        )
        .await
        .change_context(ApiError::ApiRequest)
    }

    pub async fn get_latest_build_info_raw(
        &self,
        options: SoftwareOptions,
    ) -> Result<Vec<u8>, ApiError> {
        ManagerApi::get_latest_build_info_raw(
            self.api_client.software_update_provider_config()?,
            options,
        )
        .await
        .change_context(ApiError::ApiRequest)
    }

    pub async fn get_latest_build_info(
        &self,
        options: SoftwareOptions,
    ) -> Result<BuildInfo, ApiError> {
        ManagerApi::get_latest_build_info(
            self.api_client.software_update_provider_config()?,
            options,
        )
        .await
        .change_context(ApiError::InvalidValue)
    }

    pub async fn system_info(&self, remote_manager_name: &str) -> Result<SystemInfo, ApiError> {
        ManagerApi::system_info(
            self.api_client
                .system_info_remote_manager_config(remote_manager_name)?,
        )
        .await
        .change_context(ApiError::ApiRequest)
    }
}
