//! CLI API client
//!

use error_stack::{Result, ResultExt};
use manager_model::ManagerInstanceName;

use crate::{
    api::client::{ClientConfig, ClientError, ManagerClient, RequestSenderCmds}, config::args::{ApiCommand, ManagerApiClientMode}
};

pub async fn handle_api_client_mode(args: ManagerApiClientMode) -> Result<(), ClientError> {
    let api_key = args
        .api_key()
        .change_context(ClientError::MissingConfiguration)?;
    let api_url = args
        .api_url()
        .change_context(ClientError::MissingConfiguration)?;
    let certificate = args
        .root_certificate()
        .change_context(ClientError::RootCertificateLoadingError)?;
    let manager_name = args
        .manager_name()
        .change_context(ClientError::MissingConfiguration)?;

    let config = ClientConfig {
        api_key,
        url: api_url,
        root_certificate: certificate,
    };

    let client = ManagerClient::connect(config)
        .await
        .change_context(ClientError::RemoteApiRequest)?
        .request_to(manager_name);

    match args.api_command {
        ApiCommand::AvailableInstances => {
            let list = client.get_available_instances()
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("{:#?}", list);
        }
        ApiCommand::EncryptionKey {
            encryption_key_name,
        } => {
            let key = client.get_secure_storage_encryption_key(
                ManagerInstanceName::new(encryption_key_name.clone()),
            )
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("Name: {}", encryption_key_name);
            println!("Key:  {}", key.key);
        }
        ApiCommand::LatestBuildInfo { software } => {
            // let info = ManagerApi::get_latest_build_info(&configuration, software)
            //     .await
            //     .change_context(ClientError::RemoteApiRequest)?;
            // println!("{:#?}", info);
        }
        ApiCommand::RequestUpdateSoftware {
            software,
            reboot,
            reset_data,
        } => {
            // ManagerApi::request_update_software(
            //     &configuration,
            //     software,
            //     reboot,
            //     ResetDataQueryParam { reset_data },
            // )
            // .await
            // .change_context(ClientError::RemoteApiRequest)?;
            // println!(
            //     "Update requested for {:?}, reboot: {}, reset_data: {}",
            //     software, reboot, reset_data
            // );
        }
        ApiCommand::RequestRestartBackend { reset_data } => {
            // ManagerApi::restart_backend(&configuration, ResetDataQueryParam { reset_data })
            //     .await
            //     .change_context(ClientError::RemoteApiRequest)?;
            // println!("Restart backend requested, reset_data: {}", reset_data);
        }
        ApiCommand::SystemInfo => {
            let info = client.get_system_info()
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("{:#?}", info);
        }
        ApiCommand::SoftwareInfo => {
            // let info = ManagerApi::software_info(&configuration)
            //     .await
            //     .change_context(ClientError::RemoteApiRequest)?;
            // println!("{:#?}", info);
        }
    }

    Ok(())
}
