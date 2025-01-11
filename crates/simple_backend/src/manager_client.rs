use error_stack::{Result, ResultExt};
use manager_api::{ApiKey, Configuration, ManagerApi};
use manager_model::{
    BuildInfo, ResetDataQueryParam, SoftwareInfo, SoftwareOptions, SystemInfoList,
};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tracing::{error, info};

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
    pub fn new(config: &SimpleBackendConfig) -> Result<Self, ManagerClientError> {
        let mut client = reqwest::ClientBuilder::new().tls_built_in_root_certs(false);
        if let Some(cert) = config.manager_api_root_certificate() {
            client = client.add_root_certificate(cert.clone());
        }
        let client = client
            .build()
            .change_context(ManagerClientError::ClientBuildFailed)?;

        let manager = config.manager_config().map(|c| {
            let token = ApiKey {
                prefix: None,
                key: c.api_key.to_string(),
            };

            let url = c.address.as_str().trim_end_matches('/').to_string();

            info!("Manager API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                api_key: Some(token),
                ..Configuration::default()
            }
        });

        Ok(Self { manager })
    }

    pub fn manager(&self) -> Result<&Configuration, ManagerClientError> {
        self.manager
            .as_ref()
            .ok_or(ManagerClientError::ApiUrlNotConfigured.report())
    }
}

pub struct ManagerApiManager<'a> {
    api_client: &'a ManagerApiClient,
}

impl<'a> ManagerApiManager<'a> {
    pub fn new(api_client: &'a ManagerApiClient) -> Self {
        Self { api_client }
    }

    pub async fn system_info(&self) -> Result<SystemInfoList, ManagerClientError> {
        ManagerApi::system_info_all(self.api_client.manager()?)
            .await
            .change_context(ManagerClientError::ApiRequest)
    }

    pub async fn software_info(&self) -> Result<SoftwareInfo, ManagerClientError> {
        ManagerApi::software_info(self.api_client.manager()?)
            .await
            .change_context(ManagerClientError::ApiRequest)
    }

    pub async fn get_latest_build_info(
        &self,
        options: SoftwareOptions,
    ) -> Result<BuildInfo, ManagerClientError> {
        ManagerApi::get_latest_build_info(self.api_client.manager()?, options)
            .await
            .change_context(ManagerClientError::ApiRequest)
    }

    pub async fn request_update_software(
        &self,
        options: SoftwareOptions,
        reboot: bool,
        reset_data: ResetDataQueryParam,
    ) -> Result<(), ManagerClientError> {
        ManagerApi::request_update_software(self.api_client.manager()?, options, reboot, reset_data)
            .await
            .change_context(ManagerClientError::ApiRequest)
    }

    pub async fn request_restart_or_reset_backend(
        &self,
        reset_data: ResetDataQueryParam,
    ) -> Result<(), ManagerClientError> {
        ManagerApi::restart_backend(self.api_client.manager()?, reset_data)
            .await
            .change_context(ManagerClientError::ApiRequest)
    }
}
